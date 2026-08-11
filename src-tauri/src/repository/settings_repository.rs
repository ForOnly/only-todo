use rusqlite::{params, OptionalExtension};

use crate::domain::{DockEdge, FloatDisplayMode, SettingsDto, UpdateSettingsDto, WindowBounds};
use crate::errors::AppError;
use crate::infrastructure::database::Database;

pub struct SettingsRepository;

impl SettingsRepository {
    pub fn get(db: &Database) -> Result<SettingsDto, AppError> {
        let notification_enabled = Self::get_bool(db, "notification.enabled")?.unwrap_or(true);
        let list_default_sort = Self::get_value(db, "list.default_sort")?
            .unwrap_or_else(|| r#"{"sort_by":"priority","sort_order":"desc"}"#.into());
        let float_always_on_top = Self::get_bool(db, "floating_window.always_on_top")?.unwrap_or(true);
        let float_visible_count = Self::get_u32(db, "floating_window.visible_count")?.unwrap_or(5);
        let float_auto_show = Self::get_bool(db, "floating_window.auto_show")?.unwrap_or(true);
        let float_default_mode = Self::get_display_mode_key(db, "floating_window.default_mode")?
            .unwrap_or(FloatDisplayMode::Ball);
        let autostart_enabled = Self::get_bool(db, "autostart.enabled")?.unwrap_or(false);

        Ok(SettingsDto {
            notification_enabled,
            list_default_sort,
            float_always_on_top,
            float_visible_count,
            float_auto_show,
            float_default_mode,
            autostart_enabled,
        })
    }

    pub fn update(db: &Database, dto: &UpdateSettingsDto) -> Result<SettingsDto, AppError> {
        if let Some(enabled) = dto.notification_enabled {
            Self::set_value(db, "notification.enabled", if enabled { "true" } else { "false" })?;
        }
        if let Some(sort) = &dto.list_default_sort {
            Self::set_value(db, "list.default_sort", sort)?;
        }
        if let Some(value) = dto.float_always_on_top {
            Self::set_value(
                db,
                "floating_window.always_on_top",
                if value { "true" } else { "false" },
            )?;
        }
        if let Some(count) = dto.float_visible_count {
            let clamped = count.clamp(1, 20);
            Self::set_value(db, "floating_window.visible_count", &clamped.to_string())?;
        }
        if let Some(value) = dto.float_auto_show {
            Self::set_value(db, "floating_window.auto_show", if value { "true" } else { "false" })?;
        }
        if let Some(mode) = dto.float_default_mode {
            Self::set_value(db, "floating_window.default_mode", mode.as_str())?;
        }
        if let Some(value) = dto.autostart_enabled {
            Self::set_value(db, "autostart.enabled", if value { "true" } else { "false" })?;
        }
        Self::get(db)
    }

    pub fn get_display_mode(db: &Database) -> Result<FloatDisplayMode, AppError> {
        Ok(Self::get_display_mode_key(db, "floating_window.display_mode")?
            .unwrap_or(FloatDisplayMode::Ball))
    }

    /// 用户设置的主形态（ball / panel）；docked 视为 ball
    pub fn get_home_mode(db: &Database) -> Result<FloatDisplayMode, AppError> {
        Ok(
            match Self::get_display_mode_key(db, "floating_window.default_mode")?
                .unwrap_or(FloatDisplayMode::Ball)
            {
                FloatDisplayMode::Docked => FloatDisplayMode::Ball,
                other => other,
            },
        )
    }

    pub fn set_display_mode(db: &Database, mode: FloatDisplayMode) -> Result<(), AppError> {
        Self::set_value(db, "floating_window.display_mode", mode.as_str())
    }

    pub fn get_dock_edge(db: &Database) -> Result<DockEdge, AppError> {
        let value = Self::get_value(db, "floating_window.dock_edge")?.unwrap_or_else(|| "right".into());
        DockEdge::from_str(&value).ok_or_else(|| AppError::InternalError {
            message: format!("invalid dock edge: {value}"),
        })
    }

    pub fn set_dock_edge(db: &Database, edge: DockEdge) -> Result<(), AppError> {
        Self::set_value(db, "floating_window.dock_edge", edge.as_str())
    }

    pub fn get_panel_bounds(db: &Database) -> Result<WindowBounds, AppError> {
        Self::get_bounds_key(db, "floating_window.panel_bounds", 320.0, 480.0)
    }

    pub fn set_panel_bounds(db: &Database, bounds: &WindowBounds) -> Result<(), AppError> {
        Self::set_bounds_key(db, "floating_window.panel_bounds", bounds)
    }

    pub fn get_ball_bounds(db: &Database) -> Result<WindowBounds, AppError> {
        Self::get_bounds_key(db, "floating_window.ball_bounds", 64.0, 64.0)
    }

    pub fn set_ball_bounds(db: &Database, bounds: &WindowBounds) -> Result<(), AppError> {
        Self::set_bounds_key(db, "floating_window.ball_bounds", bounds)
    }

    /// 兼容旧版 floating_window.bounds，读写 panel bounds
    pub fn get_float_bounds(db: &Database) -> Result<WindowBounds, AppError> {
        if let Some(json) = Self::get_value(db, "floating_window.panel_bounds")? {
            return serde_json::from_str(&json).map_err(|error| AppError::InternalError {
                message: format!("invalid panel_bounds: {error}"),
            });
        }
        Self::get_bounds_key(db, "floating_window.bounds", 320.0, 480.0)
    }

    pub fn set_float_bounds(db: &Database, bounds: &WindowBounds) -> Result<(), AppError> {
        Self::set_panel_bounds(db, bounds)
    }

    pub fn get_value(db: &Database, key: &str) -> Result<Option<String>, AppError> {
        db.with_conn(|conn| {
            conn.query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })
        })
    }

    pub fn set_value(db: &Database, key: &str, value: &str) -> Result<(), AppError> {
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![key, value],
            )
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })?;
            Ok(())
        })
    }

    fn get_display_mode_key(
        db: &Database,
        key: &str,
    ) -> Result<Option<FloatDisplayMode>, AppError> {
        Ok(Self::get_value(db, key)?
            .and_then(|v| FloatDisplayMode::from_str(&v)))
    }

    fn get_bounds_key(
        db: &Database,
        key: &str,
        default_w: f64,
        default_h: f64,
    ) -> Result<WindowBounds, AppError> {
        let json = Self::get_value(db, key)?.unwrap_or_else(|| {
            format!(r#"{{"x":100,"y":100,"width":{default_w},"height":{default_h}}}"#)
        });
        serde_json::from_str(&json).map_err(|error| AppError::InternalError {
            message: format!("invalid {key}: {error}"),
        })
    }

    fn set_bounds_key(db: &Database, key: &str, bounds: &WindowBounds) -> Result<(), AppError> {
        let json = serde_json::to_string(bounds).map_err(|error| AppError::InternalError {
            message: error.to_string(),
        })?;
        Self::set_value(db, key, &json)
    }

    fn get_bool(db: &Database, key: &str) -> Result<Option<bool>, AppError> {
        Ok(Self::get_value(db, key)?.map(|value| value == "true"))
    }

    fn get_u32(db: &Database, key: &str) -> Result<Option<u32>, AppError> {
        Ok(Self::get_value(db, key)?.and_then(|value| value.parse().ok()))
    }
}

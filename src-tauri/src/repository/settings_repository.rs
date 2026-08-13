use rusqlite::{params, OptionalExtension};

use crate::domain::{
    CompanionPlacement, DockEdge, HomeShape, SettingsDto, UiLocale, UiTheme, UpdateSettingsDto,
    WindowBounds,
};
use crate::errors::AppError;
use crate::infrastructure::database::Database;

pub struct SettingsRepository;

impl SettingsRepository {
    pub fn get(db: &Database) -> Result<SettingsDto, AppError> {
        let notification_enabled = Self::get_bool(db, "notification.enabled")?.unwrap_or(true);
        let list_default_sort = Self::get_value(db, "list.default_sort")?
            .unwrap_or_else(|| r#"{"sort_by":"priority","sort_order":"desc"}"#.into());
        let float_always_on_top = Self::get_bool(db, "companion.always_on_top")?.unwrap_or(true);
        let float_visible_count = Self::get_u32(db, "companion.visible_count")?.unwrap_or(5);
        let float_auto_show = Self::get_bool(db, "companion.auto_show")?.unwrap_or(true);
        let float_default_mode = Self::get_home_shape(db)?;
        let float_hover_preview = Self::get_bool(db, "companion.hover_preview")?.unwrap_or(false);
        let autostart_enabled = Self::get_bool(db, "autostart.enabled")?.unwrap_or(false);
        let ui_theme = Self::get_value(db, "ui.theme")?
            .map(|v| UiTheme::from_str(&v))
            .unwrap_or(UiTheme::System);
        let ui_locale = Self::get_value(db, "ui.locale")?
            .map(|v| UiLocale::from_str(&v))
            .unwrap_or(UiLocale::ZhCN);

        Ok(SettingsDto {
            notification_enabled,
            list_default_sort,
            float_always_on_top,
            float_visible_count,
            float_auto_show,
            float_default_mode,
            float_hover_preview,
            autostart_enabled,
            ui_theme,
            ui_locale,
        })
    }

    pub fn update(db: &Database, dto: &UpdateSettingsDto) -> Result<SettingsDto, AppError> {
        if let Some(enabled) = dto.notification_enabled {
            Self::set_value(
                db,
                "notification.enabled",
                if enabled { "true" } else { "false" },
            )?;
        }
        if let Some(sort) = &dto.list_default_sort {
            Self::set_value(db, "list.default_sort", sort)?;
        }
        if let Some(value) = dto.float_always_on_top {
            Self::set_value(
                db,
                "companion.always_on_top",
                if value { "true" } else { "false" },
            )?;
        }
        if let Some(count) = dto.float_visible_count {
            let clamped = count.clamp(1, 20);
            Self::set_value(db, "companion.visible_count", &clamped.to_string())?;
        }
        if let Some(value) = dto.float_auto_show {
            Self::set_value(
                db,
                "companion.auto_show",
                if value { "true" } else { "false" },
            )?;
        }
        if let Some(mode) = dto.float_default_mode {
            Self::set_home_shape(db, mode)?;
        }
        if let Some(value) = dto.float_hover_preview {
            Self::set_value(
                db,
                "companion.hover_preview",
                if value { "true" } else { "false" },
            )?;
        }
        if let Some(value) = dto.autostart_enabled {
            Self::set_value(
                db,
                "autostart.enabled",
                if value { "true" } else { "false" },
            )?;
        }
        if let Some(theme) = dto.ui_theme {
            Self::set_value(db, "ui.theme", theme.as_str())?;
        }
        if let Some(locale) = dto.ui_locale {
            Self::set_value(db, "ui.locale", locale.as_str())?;
        }
        Self::get(db)
    }

    pub fn get_home_shape(db: &Database) -> Result<HomeShape, AppError> {
        Ok(Self::get_value(db, "companion.home_shape")?
            .and_then(|v| HomeShape::from_str(&v))
            .unwrap_or(HomeShape::Ball))
    }

    pub fn set_home_shape(db: &Database, shape: HomeShape) -> Result<(), AppError> {
        Self::set_value(db, "companion.home_shape", shape.as_str())
    }

    pub fn get_placement(db: &Database) -> Result<CompanionPlacement, AppError> {
        Ok(Self::get_value(db, "companion.placement")?
            .and_then(|v| CompanionPlacement::from_str(&v))
            .unwrap_or(CompanionPlacement::Free))
    }

    pub fn set_placement(db: &Database, placement: CompanionPlacement) -> Result<(), AppError> {
        Self::set_value(db, "companion.placement", placement.as_str())
    }

    pub fn get_dock_edge(db: &Database) -> Result<DockEdge, AppError> {
        let value = Self::get_value(db, "companion.dock_edge")?.unwrap_or_else(|| "right".into());
        Ok(DockEdge::from_str(&value).unwrap_or(DockEdge::Right))
    }

    pub fn set_dock_edge(db: &Database, edge: DockEdge) -> Result<(), AppError> {
        Self::set_value(db, "companion.dock_edge", edge.as_str())?;
        Self::set_has_docked(db, true)
    }

    pub fn get_has_docked(db: &Database) -> Result<bool, AppError> {
        Ok(Self::get_bool(db, "companion.has_docked")?.unwrap_or(false))
    }

    pub fn set_has_docked(db: &Database, value: bool) -> Result<(), AppError> {
        Self::set_value(
            db,
            "companion.has_docked",
            if value { "true" } else { "false" },
        )
    }

    pub fn get_dock_y(db: &Database) -> Result<f64, AppError> {
        Ok(Self::get_value(db, "companion.dock_y")?
            .and_then(|v| v.parse().ok())
            .unwrap_or(100.0))
    }

    pub fn set_dock_y(db: &Database, y: f64) -> Result<(), AppError> {
        Self::set_value(db, "companion.dock_y", &format!("{y}"))
    }

    pub fn get_panel_bounds(db: &Database) -> Result<WindowBounds, AppError> {
        Self::get_bounds_key(db, "companion.panel_bounds", 320.0, 480.0)
    }

    pub fn set_panel_bounds(db: &Database, bounds: &WindowBounds) -> Result<(), AppError> {
        Self::set_bounds_key(db, "companion.panel_bounds", bounds)
    }

    pub fn get_ball_pos(db: &Database) -> Result<WindowBounds, AppError> {
        let mut bounds = Self::get_bounds_key(db, "companion.ball_pos", 64.0, 64.0)?;
        bounds.width = 64.0;
        bounds.height = 64.0;
        Ok(bounds)
    }

    pub fn set_ball_pos(db: &Database, bounds: &WindowBounds) -> Result<(), AppError> {
        let mut stored = bounds.clone();
        stored.width = 64.0;
        stored.height = 64.0;
        Self::set_bounds_key(db, "companion.ball_pos", &stored)
    }

    pub fn get_hover_preview(db: &Database) -> Result<bool, AppError> {
        Ok(Self::get_bool(db, "companion.hover_preview")?.unwrap_or(false))
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

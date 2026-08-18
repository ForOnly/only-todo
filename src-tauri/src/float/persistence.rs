//! 伴侣位置/尺寸持久化
use tauri::AppHandle;

use crate::domain::{CompanionPlacement, CompanionVisibility, HomeShape, PanelMode, WindowBounds};
use crate::errors::AppError;
use crate::repository::settings_repository::SettingsRepository;

use super::geometry::{
    current_window_bounds, inner_window_size, panel_from_ball, snap_ball_to_work_area,
    snap_handle_to_work_area, work_area_logical, HANDLE_HEIGHT,
};
use super::session::{app_state, host_state, lock_runtime};
use super::surfaces::{apply_position, body_window, chrome_window};

pub(crate) fn free_body_bounds(
    app: &AppHandle,
    db: &crate::infrastructure::database::Database,
    home: HomeShape,
    body: &tauri::WebviewWindow,
    chrome: &tauri::WebviewWindow,
) -> Result<WindowBounds, AppError> {
    match home {
        HomeShape::Panel => SettingsRepository::get_panel_bounds(db),
        HomeShape::Ball => {
            let temp = {
                let host = host_state(app)?;
                let bounds = lock_runtime(&host)?.temp_body_bounds.clone();
                bounds
            };
            if let Some(bounds) = temp {
                return Ok(bounds);
            }
            let work = work_area_logical(body).or_else(|_| work_area_logical(chrome))?;
            let ball = SettingsRepository::get_ball_pos(db)?;
            let panel = SettingsRepository::get_panel_bounds(db)?;
            Ok(panel_from_ball(work, &ball, &panel))
        }
    }
}

pub(crate) fn persist_panel_size(
    db: &crate::infrastructure::database::Database,
    width: f64,
    height: f64,
) -> Result<(), AppError> {
    let mut saved = SettingsRepository::get_panel_bounds(db)?;
    saved.width = width;
    saved.height = height;
    SettingsRepository::set_panel_bounds(db, &saved)
}

pub(crate) fn capture_temp_body(app: &AppHandle) -> Result<(), AppError> {
    let Some(body) = body_window(app) else {
        return Ok(());
    };
    if !body.is_visible().unwrap_or(false) {
        return Ok(());
    }
    let outer = current_window_bounds(&body)?;
    let (width, height) = inner_window_size(&body)?;
    let host = host_state(app)?;
    lock_runtime(&host)?.temp_body_bounds = Some(WindowBounds {
        x: outer.x,
        y: outer.y,
        width,
        height,
    });
    Ok(())
}

pub(crate) fn apply_home_shape_change(app: &AppHandle) -> Result<(), AppError> {
    if SettingsRepository::get_placement(&app_state(app)?.db)? == CompanionPlacement::Docked {
        return Ok(());
    }
    let home = SettingsRepository::get_home_shape(&app_state(app)?.db)?;
    match home {
        HomeShape::Panel => {
            if let Some(body) = body_window(app) {
                if body.is_visible().unwrap_or(false) {
                    persist_body_bounds(app)?;
                } else {
                    let host = host_state(app)?;
                    let temp = lock_runtime(&host)?.temp_body_bounds.clone();
                    if let Some(bounds) = temp {
                        SettingsRepository::set_panel_bounds(&app_state(app)?.db, &bounds)?;
                    }
                }
            }
            let host = host_state(app)?;
            let mut rt = lock_runtime(&host)?;
            rt.temp_body_bounds = None;
            if rt.visibility == CompanionVisibility::Shown {
                rt.panel_mode = PanelMode::Pinned;
            }
        }
        HomeShape::Ball => {
            persist_body_size_only(app).ok();
            let host = host_state(app)?;
            let mut rt = lock_runtime(&host)?;
            rt.temp_body_bounds = None;
            if rt.visibility == CompanionVisibility::Shown {
                rt.panel_mode = PanelMode::Closed;
            }
        }
    }
    Ok(())
}

pub(crate) fn persist_body_bounds(app: &AppHandle) -> Result<(), AppError> {
    let Some(body) = body_window(app) else {
        return Ok(());
    };
    if !body.is_visible().unwrap_or(false) {
        return Ok(());
    }
    let outer = current_window_bounds(&body)?;
    let (width, height) = inner_window_size(&body)?;
    SettingsRepository::set_panel_bounds(
        &app_state(app)?.db,
        &WindowBounds {
            x: outer.x,
            y: outer.y,
            width,
            height,
        },
    )
}

pub(crate) fn persist_body_size_only(app: &AppHandle) -> Result<(), AppError> {
    let Some(body) = body_window(app) else {
        return Ok(());
    };
    if !body.is_visible().unwrap_or(false) {
        return Ok(());
    }
    let (width, height) = inner_window_size(&body)?;
    let db = &app_state(app)?.db;
    let mut saved = SettingsRepository::get_panel_bounds(db)?;
    saved.width = width;
    saved.height = height;
    SettingsRepository::set_panel_bounds(db, &saved)
}

pub(crate) fn follow_chrome_y_to_body(app: &AppHandle) -> Result<(), AppError> {
    if SettingsRepository::get_placement(&app_state(app)?.db)? != CompanionPlacement::Docked {
        return Ok(());
    }
    let Some(body) = body_window(app) else {
        return Ok(());
    };
    let Some(chrome) = chrome_window(app) else {
        return Ok(());
    };
    if !body.is_visible().unwrap_or(false) || !chrome.is_visible().unwrap_or(false) {
        return Ok(());
    }
    let body_b = current_window_bounds(&body)?;
    let work = work_area_logical(&chrome)?;
    let edge = SettingsRepository::get_dock_edge(&app_state(app)?.db)?;
    let y = body_b.y + body_b.height / 2.0 - HANDLE_HEIGHT / 2.0;
    let actual = current_window_bounds(&chrome)?;
    let snapped = snap_handle_to_work_area(work, edge, y, &actual);
    apply_position(&chrome, snapped.x, snapped.y)
}

pub(crate) fn persist_chrome_ball(app: &AppHandle) -> Result<(), AppError> {
    let Some(chrome) = chrome_window(app) else {
        return Ok(());
    };
    if !chrome.is_visible().unwrap_or(false) {
        return Ok(());
    }
    let current = current_window_bounds(&chrome)?;
    let work = work_area_logical(&chrome)?;
    let snapped = snap_ball_to_work_area(work, current.x, current.y, Some(&current));
    SettingsRepository::set_ball_pos(&app_state(app)?.db, &snapped)
}

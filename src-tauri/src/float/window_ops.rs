//! 伴侣窗口表面应用（chrome/body 显隐与几何）
use std::sync::atomic::Ordering;

use tauri::{AppHandle, Emitter};

use crate::domain::{
    derive_chrome, panel_visible, ChromeKind, CompanionPlacement, CompanionSession,
    CompanionVisibility, DockEdge, PanelMode,
};
use crate::errors::AppError;
use crate::repository::settings_repository::SettingsRepository;
use crate::services::settings_service::SettingsService;

use super::geometry::{
    ball_visual_hit, body_beside_handle, clamp_panel_in_work, current_window_bounds, handle_bounds,
    os_primary_mouse_down, snap_ball_to_work_area, snap_handle_to_work_area, strip_visual_hit,
    work_area_logical,
};
use super::persistence::free_body_bounds;
use super::session::{app_state, host_state, lock_runtime, snapshot};
use super::surfaces::{
    apply_bounds, apply_position, body_window, chrome_window, hide_window, set_always_on_top,
};
use super::SESSION_EVENT;

pub(crate) fn apply_always_on_top(app: &AppHandle, value: bool) {
    let Ok(host) = host_state(app) else {
        return;
    };
    let Ok(mut last) = host.last_aot.lock() else {
        return;
    };
    if *last == Some(value) {
        return;
    }
    *last = Some(value);
    set_always_on_top(app, value);
}

pub(crate) fn cache_chrome_hit(app: &AppHandle, kind: ChromeKind, edge: DockEdge) {
    if let Ok(host) = host_state(app) {
        if let Ok(mut hit) = host.chrome_hit.lock() {
            *hit = match kind {
                ChromeKind::Hidden => None,
                _ => Some((kind, edge)),
            };
        }
    }
}

pub(crate) fn set_chrome_ignore(app: &AppHandle, chrome: &tauri::WebviewWindow, ignore: bool) {
    let Ok(host) = host_state(app) else {
        return;
    };
    if host.chrome_ignore.swap(ignore, Ordering::SeqCst) == ignore {
        return;
    }
    let _ = chrome.set_ignore_cursor_events(ignore);
}

pub fn update_chrome_click_through(app: &AppHandle) -> Result<(), AppError> {
    let Some(chrome) = chrome_window(app) else {
        return Ok(());
    };
    if !chrome.is_visible().unwrap_or(false) {
        set_chrome_ignore(app, &chrome, false);
        return Ok(());
    }
    if os_primary_mouse_down() {
        set_chrome_ignore(app, &chrome, false);
        return Ok(());
    }
    let hit = {
        let host = host_state(app)?;
        host.chrome_hit.lock().ok().and_then(|guard| *guard)
    };
    let Some((kind, edge)) = hit else {
        set_chrome_ignore(app, &chrome, false);
        return Ok(());
    };
    let scale = chrome.scale_factor().unwrap_or(1.0);
    let cursor = chrome
        .cursor_position()
        .map_err(|error| AppError::InternalError {
            message: error.to_string(),
        })?;
    let cx = cursor.x / scale;
    let cy = cursor.y / scale;
    let bounds = current_window_bounds(&chrome)?;
    let inside = match kind {
        ChromeKind::Ball => ball_visual_hit(&bounds, cx, cy),
        ChromeKind::Strip => strip_visual_hit(&bounds, edge, cx, cy),
        ChromeKind::Hidden => false,
    };
    set_chrome_ignore(app, &chrome, !inside);
    Ok(())
}

pub(crate) fn apply_and_emit(app: &AppHandle) -> Result<CompanionSession, AppError> {
    apply_surfaces(app)?;
    let session = snapshot(app)?;
    let _ = app.emit(SESSION_EVENT, &session);
    super::update_tray_tooltip(app);
    Ok(session)
}

pub(crate) fn apply_surfaces(app: &AppHandle) -> Result<(), AppError> {
    let host = host_state(app)?;
    host.applying.store(true, Ordering::SeqCst);
    let gen = host.apply_gen.fetch_add(1, Ordering::SeqCst) + 1;
    let result = apply_surfaces_inner(app);
    let app_clone = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(150));
        if let Ok(host) = host_state(&app_clone) {
            if host.apply_gen.load(Ordering::SeqCst) == gen {
                host.applying.store(false, Ordering::SeqCst);
            }
        }
    });
    result
}

fn apply_surfaces_inner(app: &AppHandle) -> Result<(), AppError> {
    let state = app_state(app)?;
    let db = &state.db;
    let always_on_top = SettingsService::float_always_on_top(db).unwrap_or(true);
    apply_always_on_top(app, always_on_top);

    let (visibility, panel_mode, want_focus) = {
        let host = host_state(app)?;
        let rt = lock_runtime(&host)?;
        (
            rt.visibility,
            rt.panel_mode,
            host.focus_body_once.swap(false, Ordering::SeqCst),
        )
    };
    let placement = SettingsRepository::get_placement(db)?;
    let home = SettingsRepository::get_home_shape(db)?;
    let chrome_kind = derive_chrome(visibility, placement, home, panel_mode);
    let body_on = visibility == CompanionVisibility::Shown && panel_visible(placement, panel_mode);

    let Some(chrome) = chrome_window(app) else {
        return Ok(());
    };
    let Some(body) = body_window(app) else {
        return Ok(());
    };

    if visibility == CompanionVisibility::Hidden {
        hide_window(&chrome)?;
        hide_window(&body)?;
        cache_chrome_hit(app, ChromeKind::Hidden, DockEdge::Right);
        return Ok(());
    }

    if body_on {
        match placement {
            CompanionPlacement::Free => {
                hide_window(&chrome)?;
                let mut bounds = free_body_bounds(app, db, home, &body, &chrome)?;
                if let Ok(work) = work_area_logical(&body).or_else(|_| work_area_logical(&chrome)) {
                    clamp_panel_in_work(work, &mut bounds);
                }
                apply_bounds(&body, &bounds, true)?;
                cache_chrome_hit(app, ChromeKind::Hidden, DockEdge::Right);
                if want_focus && panel_mode == PanelMode::Pinned {
                    let _ = body.set_focus();
                }
            }
            CompanionPlacement::Docked => {
                let work = work_area_logical(&chrome).or_else(|_| work_area_logical(&body))?;
                let edge = SettingsRepository::get_dock_edge(db)?;
                let y = SettingsRepository::get_dock_y(db)?;
                let requested = handle_bounds(work, edge, y);
                apply_bounds(&chrome, &requested, true)?;
                let h_bounds = match current_window_bounds(&chrome) {
                    Ok(actual) => {
                        let snapped = snap_handle_to_work_area(work, edge, y, &actual);
                        if (snapped.x - actual.x).abs() > 0.5 || (snapped.y - actual.y).abs() > 0.5
                        {
                            apply_position(&chrome, snapped.x, snapped.y)?;
                        }
                        snapped
                    }
                    Err(_) => requested,
                };
                let panel = SettingsRepository::get_panel_bounds(db)?;
                let b_bounds = body_beside_handle(work, edge, &h_bounds, &panel);
                apply_bounds(&body, &b_bounds, true)?;
                cache_chrome_hit(app, ChromeKind::Strip, edge);
                set_chrome_ignore(app, &chrome, false);
                if want_focus && panel_mode == PanelMode::Pinned {
                    let _ = body.set_focus();
                }
            }
        }
        return Ok(());
    }

    hide_window(&body)?;
    match chrome_kind {
        ChromeKind::Hidden => {
            hide_window(&chrome)?;
            cache_chrome_hit(app, ChromeKind::Hidden, DockEdge::Right);
        }
        ChromeKind::Ball => {
            let work = work_area_logical(&chrome).or_else(|_| work_area_logical(&body))?;
            let saved = SettingsRepository::get_ball_pos(db)?;
            let requested = snap_ball_to_work_area(work, saved.x, saved.y, None);
            apply_bounds(&chrome, &requested, true)?;
            if let Ok(actual) = current_window_bounds(&chrome) {
                let snapped = snap_ball_to_work_area(work, requested.x, requested.y, Some(&actual));
                if (snapped.x - actual.x).abs() > 0.5 || (snapped.y - actual.y).abs() > 0.5 {
                    apply_position(&chrome, snapped.x, snapped.y)?;
                }
            }
            cache_chrome_hit(app, ChromeKind::Ball, DockEdge::Right);
            set_chrome_ignore(app, &chrome, false);
        }
        ChromeKind::Strip => {
            let work = work_area_logical(&chrome).or_else(|_| work_area_logical(&body))?;
            let edge = SettingsRepository::get_dock_edge(db)?;
            let y = SettingsRepository::get_dock_y(db)?;
            let requested = handle_bounds(work, edge, y);
            apply_bounds(&chrome, &requested, true)?;
            if let Ok(actual) = current_window_bounds(&chrome) {
                let snapped = snap_handle_to_work_area(work, edge, y, &actual);
                if (snapped.x - actual.x).abs() > 0.5 || (snapped.y - actual.y).abs() > 0.5 {
                    apply_position(&chrome, snapped.x, snapped.y)?;
                }
            }
            cache_chrome_hit(app, ChromeKind::Strip, edge);
            set_chrome_ignore(app, &chrome, false);
        }
    }
    Ok(())
}

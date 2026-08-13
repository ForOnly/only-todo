use std::sync::atomic::Ordering;

use tauri::{AppHandle, Emitter};

use crate::domain::{
    BodyView, ChromeKind, CompanionDragEndResult, CompanionPlacement, CompanionSession,
    CompanionSurface, CompanionVisibility, DockEdge, HomeShape, PanelMode, WindowBounds,
};
use crate::errors::AppError;
use crate::repository::settings_repository::SettingsRepository;
use crate::services::settings_service::SettingsService;

use super::geometry::{
    clamp_panel_in_work, closer_vertical_edge, current_window_bounds, detect_nearest_edge,
    dock_drag_outcome, dock_threshold, os_primary_mouse_down, panel_from_ball,
    snap_ball_to_work_area, work_area_logical, DockDragOutcome, BALL_SIZE, HANDLE_HEIGHT,
    PANEL_DEFAULT_H, PANEL_DEFAULT_W,
};
use super::persistence::{
    apply_home_shape_change, capture_temp_body, follow_chrome_y_to_body, persist_body_bounds,
    persist_body_size_only, persist_chrome_ball, persist_panel_size,
};
use super::session::{
    app_state, companion_is_shown, companion_lost, host_state, lock_runtime, snapshot,
};
use super::surfaces::{body_window, chrome_window};
use super::window_ops::{apply_always_on_top, apply_and_emit};
use super::SESSION_EVENT;

pub use super::session::FloatHost;

impl FloatHost {
    pub fn show(app: &AppHandle) -> Result<CompanionSession, AppError> {
        if companion_is_shown(app) && !companion_lost(app) {
            {
                let host = host_state(app)?;
                lock_runtime(&host)?.visibility = CompanionVisibility::Shown;
            }
            return snapshot(app);
        }
        {
            let state = app_state(app)?;
            let db = &state.db;
            let placement = SettingsRepository::get_placement(db)?;
            let home = SettingsRepository::get_home_shape(db)?;
            let host = host_state(app)?;
            let mut rt = lock_runtime(&host)?;
            rt.visibility = CompanionVisibility::Shown;
            rt.panel_mode = match placement {
                CompanionPlacement::Docked => PanelMode::Closed,
                CompanionPlacement::Free => match home {
                    HomeShape::Panel => PanelMode::Pinned,
                    HomeShape::Ball => PanelMode::Closed,
                },
            };
            if home == HomeShape::Ball {
                rt.temp_body_bounds = None;
            }
        }
        apply_and_emit(app)
    }

    pub fn hide(app: &AppHandle) -> Result<CompanionSession, AppError> {
        let state = app_state(app)?;
        let db = &state.db;
        let home = SettingsRepository::get_home_shape(db)?;
        let docked = SettingsRepository::get_placement(db)? == CompanionPlacement::Docked;
        if home == HomeShape::Ball {
            persist_body_size_only(app).ok();
        } else if !docked {
            persist_body_bounds(app).ok();
        }
        {
            let host = host_state(app)?;
            let mut rt = lock_runtime(&host)?;
            rt.visibility = CompanionVisibility::Hidden;
            if docked || home == HomeShape::Ball {
                rt.panel_mode = PanelMode::Closed;
                rt.temp_body_bounds = None;
            }
        }
        apply_and_emit(app)
    }

    pub fn toggle(app: &AppHandle) -> Result<CompanionSession, AppError> {
        if companion_is_shown(app) && !companion_lost(app) {
            Self::hide(app)
        } else {
            Self::show(app)
        }
    }

    pub fn click_chrome(app: &AppHandle) -> Result<CompanionSession, AppError> {
        cancel_cluster_timers(app);
        let state = app_state(app)?;
        let db = &state.db;
        let placement = SettingsRepository::get_placement(db)?;
        let home = SettingsRepository::get_home_shape(db)?;
        match placement {
            CompanionPlacement::Docked => {
                let host = host_state(app)?;
                let mut rt = lock_runtime(&host)?;
                rt.visibility = CompanionVisibility::Shown;
                rt.panel_mode = match rt.panel_mode {
                    PanelMode::Pinned => PanelMode::Closed,
                    PanelMode::Closed => {
                        host.focus_body_once.store(true, Ordering::SeqCst);
                        PanelMode::Pinned
                    }
                    PanelMode::Preview => PanelMode::Pinned,
                };
            }
            CompanionPlacement::Free => {
                if home != HomeShape::Ball {
                    return snapshot(app);
                }
                persist_chrome_ball(app)?;
                let Some(chrome) = chrome_window(app) else {
                    return snapshot(app);
                };
                let work = work_area_logical(&chrome)?;
                let ball = match current_window_bounds(&chrome) {
                    Ok(bounds) => bounds,
                    Err(_) => SettingsRepository::get_ball_pos(db)?,
                };
                let panel = SettingsRepository::get_panel_bounds(db)?;
                let placed = panel_from_ball(work, &ball, &panel);
                persist_panel_size(db, placed.width, placed.height)?;
                let host = host_state(app)?;
                host.focus_body_once.store(true, Ordering::SeqCst);
                let mut rt = lock_runtime(&host)?;
                rt.visibility = CompanionVisibility::Shown;
                rt.panel_mode = PanelMode::Pinned;
                rt.temp_body_bounds = Some(placed);
            }
        }
        apply_and_emit(app)
    }

    pub fn minimize(app: &AppHandle) -> Result<CompanionSession, AppError> {
        cancel_cluster_timers(app);
        let state = app_state(app)?;
        let db = &state.db;
        let placement = SettingsRepository::get_placement(db)?;
        let home = SettingsRepository::get_home_shape(db)?;
        match placement {
            CompanionPlacement::Docked => {
                let host = host_state(app)?;
                lock_runtime(&host)?.panel_mode = PanelMode::Closed;
            }
            CompanionPlacement::Free => match home {
                HomeShape::Ball => {
                    persist_body_size_only(app).ok();
                    let host = host_state(app)?;
                    let mut rt = lock_runtime(&host)?;
                    rt.panel_mode = PanelMode::Closed;
                    rt.temp_body_bounds = None;
                }
                HomeShape::Panel => {
                    persist_body_bounds(app).ok();
                    dock_from_body_minimize(app)?;
                    let host = host_state(app)?;
                    let mut rt = lock_runtime(&host)?;
                    rt.visibility = CompanionVisibility::Shown;
                    rt.panel_mode = PanelMode::Closed;
                }
            },
        }
        apply_and_emit(app)
    }

    /// 面板 ×：一律收成贴边条（含圆球家临时板，不回球、不进托盘）
    pub fn collapse_to_strip(app: &AppHandle) -> Result<CompanionSession, AppError> {
        cancel_cluster_timers(app);
        let state = app_state(app)?;
        let db = &state.db;
        let placement = SettingsRepository::get_placement(db)?;
        match placement {
            CompanionPlacement::Docked => {
                let host = host_state(app)?;
                lock_runtime(&host)?.panel_mode = PanelMode::Closed;
            }
            CompanionPlacement::Free => {
                persist_body_bounds(app).ok();
                dock_from_body_minimize(app)?;
                let host = host_state(app)?;
                let mut rt = lock_runtime(&host)?;
                rt.visibility = CompanionVisibility::Shown;
                rt.panel_mode = PanelMode::Closed;
                rt.temp_body_bounds = None;
            }
        }
        apply_and_emit(app)
    }

    pub fn drag_ended(
        app: &AppHandle,
        which: CompanionSurface,
    ) -> Result<CompanionDragEndResult, AppError> {
        if os_primary_mouse_down() {
            return Ok(CompanionDragEndResult {
                still_dragging: true,
                session: snapshot(app)?,
            });
        }
        cancel_cluster_timers(app);
        match which {
            CompanionSurface::Chrome => drag_chrome_ended(app)?,
            CompanionSurface::Body => drag_body_ended(app)?,
        }
        Ok(CompanionDragEndResult {
            still_dragging: false,
            session: apply_and_emit(app)?,
        })
    }

    pub fn pointer_cluster(
        app: &AppHandle,
        surface: CompanionSurface,
        inside: Option<bool>,
        focused: Option<bool>,
    ) -> Result<CompanionSession, AppError> {
        let hover = SettingsRepository::get_hover_preview(&app_state(app)?.db)?;
        {
            let host = host_state(app)?;
            let mut cluster = host.cluster.lock().map_err(|_| AppError::InternalError {
                message: "float host lock poisoned".into(),
            })?;
            match surface {
                CompanionSurface::Chrome => {
                    if let Some(value) = inside {
                        cluster.chrome_inside = value;
                    }
                }
                CompanionSurface::Body => {
                    if let Some(value) = inside {
                        cluster.body_inside = value;
                    }
                    if let Some(value) = focused {
                        cluster.body_focused = value;
                    }
                }
            }
        }

        if !hover {
            return snapshot(app);
        }
        if SettingsRepository::get_placement(&app_state(app)?.db)? != CompanionPlacement::Docked {
            return snapshot(app);
        }

        let (visibility, panel_mode) = {
            let host = host_state(app)?;
            let rt = lock_runtime(&host)?;
            (rt.visibility, rt.panel_mode)
        };
        let (chrome_inside, body_inside, body_focused) = {
            let host = host_state(app)?;
            let cluster = host.cluster.lock().map_err(|_| AppError::InternalError {
                message: "float host lock poisoned".into(),
            })?;
            (
                cluster.chrome_inside,
                cluster.body_inside,
                cluster.body_focused,
            )
        };

        if visibility != CompanionVisibility::Shown || panel_mode == PanelMode::Pinned {
            return snapshot(app);
        }

        if panel_mode == PanelMode::Closed && chrome_inside {
            schedule_preview(app.clone());
        } else if panel_mode == PanelMode::Preview
            && !chrome_inside
            && !body_inside
            && !body_focused
        {
            schedule_unpreview(app.clone());
        } else if chrome_inside || body_inside || body_focused {
            cancel_cluster_timers(app);
        }

        snapshot(app)
    }

    pub fn open_view(app: &AppHandle, view: BodyView) -> Result<CompanionSession, AppError> {
        {
            let host = host_state(app)?;
            lock_runtime(&host)?.body_view = view;
        }
        apply_and_emit(app)
    }

    pub fn publish(app: &AppHandle) -> Result<CompanionSession, AppError> {
        let session = snapshot(app)?;
        let _ = app.emit(SESSION_EVENT, &session);
        super::update_tray_tooltip(app);
        Ok(session)
    }

    pub fn on_settings_updated(app: &AppHandle, home_changed: bool) -> Result<(), AppError> {
        let always = SettingsService::float_always_on_top(&app_state(app)?.db).unwrap_or(true);
        apply_always_on_top(app, always);
        let hover = SettingsRepository::get_hover_preview(&app_state(app)?.db)?;
        let mut need_apply = false;
        {
            let host = host_state(app)?;
            let mut rt = lock_runtime(&host)?;
            if !hover && rt.panel_mode == PanelMode::Preview {
                rt.panel_mode = PanelMode::Closed;
                need_apply = true;
            }
        }
        if home_changed {
            let docked = SettingsRepository::get_placement(&app_state(app)?.db)?
                == CompanionPlacement::Docked;
            apply_home_shape_change(app)?;
            if !docked {
                need_apply = true;
            }
        }
        if need_apply && companion_is_shown(app) {
            apply_and_emit(app)?;
        } else {
            let _ = Self::publish(app);
        }
        Ok(())
    }

    pub fn persist_body_if_needed(app: &AppHandle) -> Result<(), AppError> {
        let host = host_state(app)?;
        if host.is_applying() {
            return Ok(());
        }
        let state = app_state(app)?;
        let db = &state.db;
        let placement = SettingsRepository::get_placement(db)?;
        let home = SettingsRepository::get_home_shape(db)?;
        if placement != CompanionPlacement::Free {
            follow_chrome_y_to_body(app).ok();
            return Ok(());
        }
        if home == HomeShape::Ball {
            capture_temp_body(app)?;
            persist_body_size_only(app)
        } else {
            persist_body_bounds(app)
        }
    }

    pub fn persist_chrome_if_ball(app: &AppHandle) -> Result<(), AppError> {
        let host = host_state(app)?;
        if host.is_applying() {
            return Ok(());
        }
        let session = snapshot(app)?;
        if session.chrome != ChromeKind::Ball {
            return Ok(());
        }
        persist_chrome_ball(app)
    }
}

fn cancel_cluster_timers(app: &AppHandle) {
    if let Ok(host) = host_state(app) {
        if let Ok(mut cluster) = host.cluster.lock() {
            cluster.preview_gen = cluster.preview_gen.wrapping_add(1);
            cluster.unpreview_gen = cluster.unpreview_gen.wrapping_add(1);
        }
    }
}

fn schedule_preview(app: AppHandle) {
    let Ok(host) = host_state(&app) else {
        return;
    };
    let gen = {
        let Ok(mut cluster) = host.cluster.lock() else {
            return;
        };
        cluster.unpreview_gen = cluster.unpreview_gen.wrapping_add(1);
        cluster.preview_gen = cluster.preview_gen.wrapping_add(1);
        cluster.preview_gen
    };
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(400));
        let Ok(host) = host_state(&app) else {
            return;
        };
        let still = host
            .cluster
            .lock()
            .map(|cluster| cluster.preview_gen == gen && cluster.chrome_inside)
            .unwrap_or(false);
        if !still {
            return;
        }
        {
            let Ok(mut rt) = host.runtime.lock() else {
                return;
            };
            if rt.visibility != CompanionVisibility::Shown || rt.panel_mode != PanelMode::Closed {
                return;
            }
            rt.panel_mode = PanelMode::Preview;
        }
        let _ = apply_and_emit(&app);
    });
}

fn schedule_unpreview(app: AppHandle) {
    let Ok(host) = host_state(&app) else {
        return;
    };
    let gen = {
        let Ok(mut cluster) = host.cluster.lock() else {
            return;
        };
        cluster.preview_gen = cluster.preview_gen.wrapping_add(1);
        cluster.unpreview_gen = cluster.unpreview_gen.wrapping_add(1);
        cluster.unpreview_gen
    };
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(300));
        let Ok(host) = host_state(&app) else {
            return;
        };
        let still = host
            .cluster
            .lock()
            .map(|cluster| {
                cluster.unpreview_gen == gen
                    && !cluster.chrome_inside
                    && !cluster.body_inside
                    && !cluster.body_focused
            })
            .unwrap_or(false);
        if !still {
            return;
        }
        {
            let Ok(mut rt) = host.runtime.lock() else {
                return;
            };
            if rt.panel_mode != PanelMode::Preview {
                return;
            }
            rt.panel_mode = PanelMode::Closed;
        }
        let _ = apply_and_emit(&app);
    });
}

fn dock_from_body_minimize(app: &AppHandle) -> Result<(), AppError> {
    let state = app_state(app)?;
    let db = &state.db;
    let Some(body) = body_window(app) else {
        return Ok(());
    };
    let bounds = current_window_bounds(&body).ok();
    let edge = if SettingsRepository::get_has_docked(db)? {
        SettingsRepository::get_dock_edge(db)?
    } else {
        closer_vertical_edge(&body)
    };
    let y = bounds
        .as_ref()
        .map(|b| b.y + b.height / 2.0 - HANDLE_HEIGHT / 2.0)
        .unwrap_or_else(|| SettingsRepository::get_dock_y(db).unwrap_or(100.0));
    persist_body_bounds(app).ok();
    SettingsRepository::set_placement(db, CompanionPlacement::Docked)?;
    SettingsRepository::set_dock_edge(db, edge)?;
    SettingsRepository::set_dock_y(db, y)?;
    Ok(())
}

fn dock_at(app: &AppHandle, edge: DockEdge, y: f64) -> Result<(), AppError> {
    let db = &app_state(app)?.db;
    SettingsRepository::set_placement(db, CompanionPlacement::Docked)?;
    SettingsRepository::set_dock_edge(db, edge)?;
    SettingsRepository::set_dock_y(db, y)?;
    let host = host_state(app)?;
    let mut rt = lock_runtime(&host)?;
    rt.visibility = CompanionVisibility::Shown;
    rt.panel_mode = PanelMode::Closed;
    rt.temp_body_bounds = None;
    Ok(())
}

fn undock_to_home(app: &AppHandle, drop: &WindowBounds) -> Result<(), AppError> {
    let state = app_state(app)?;
    let db = &state.db;
    let home = SettingsRepository::get_home_shape(db)?;
    let Some(chrome) = chrome_window(app) else {
        return Ok(());
    };
    let work = work_area_logical(&chrome).or_else(|_| {
        body_window(app)
            .ok_or_else(|| AppError::InternalError {
                message: "no companion window".into(),
            })
            .and_then(|body| work_area_logical(&body))
    })?;
    SettingsRepository::set_placement(db, CompanionPlacement::Free)?;
    match home {
        HomeShape::Ball => {
            let x = drop.x + drop.width / 2.0 - BALL_SIZE / 2.0;
            let y = drop.y + drop.height / 2.0 - BALL_SIZE / 2.0;
            let ball = snap_ball_to_work_area(work, x, y, None);
            SettingsRepository::set_ball_pos(db, &ball)?;
            let host = host_state(app)?;
            let mut rt = lock_runtime(&host)?;
            rt.panel_mode = PanelMode::Closed;
            rt.temp_body_bounds = None;
        }
        HomeShape::Panel => {
            let mut panel = SettingsRepository::get_panel_bounds(db).unwrap_or(WindowBounds {
                x: drop.x,
                y: drop.y,
                width: PANEL_DEFAULT_W,
                height: PANEL_DEFAULT_H,
            });
            panel.x = drop.x + drop.width / 2.0 - panel.width / 2.0;
            panel.y = drop.y + drop.height / 2.0 - panel.height / 2.0;
            clamp_panel_in_work(work, &mut panel);
            SettingsRepository::set_panel_bounds(db, &panel)?;
            let host = host_state(app)?;
            lock_runtime(&host)?.panel_mode = PanelMode::Pinned;
        }
    }
    Ok(())
}

fn drag_chrome_ended(app: &AppHandle) -> Result<(), AppError> {
    let state = app_state(app)?;
    let db = &state.db;
    let Some(chrome) = chrome_window(app) else {
        return Ok(());
    };
    let placement = SettingsRepository::get_placement(db)?;
    if placement == CompanionPlacement::Docked {
        let edge = SettingsRepository::get_dock_edge(db)?;
        let current = current_window_bounds(&chrome)?;
        let work = work_area_logical(&chrome)?;
        let threshold = dock_threshold(current.width);
        match dock_drag_outcome(&current, work, edge, threshold) {
            DockDragOutcome::StayCurrent => {
                SettingsRepository::set_dock_y(db, current.y)?;
            }
            DockDragOutcome::Switch(next) => {
                SettingsRepository::set_dock_edge(db, next)?;
                SettingsRepository::set_dock_y(db, current.y)?;
            }
            DockDragOutcome::Undock => undock_to_home(app, &current)?,
        }
        return Ok(());
    }

    persist_chrome_ball(app)?;
    if let Some(edge) = detect_nearest_edge(&chrome) {
        let bounds = current_window_bounds(&chrome)?;
        dock_at(
            app,
            edge,
            bounds.y + bounds.height / 2.0 - HANDLE_HEIGHT / 2.0,
        )?;
    }
    Ok(())
}

fn drag_body_ended(app: &AppHandle) -> Result<(), AppError> {
    let state = app_state(app)?;
    let db = &state.db;
    let Some(body) = body_window(app) else {
        return Ok(());
    };
    let placement = SettingsRepository::get_placement(db)?;
    if placement == CompanionPlacement::Docked {
        persist_body_size_only(app)?;
        let current = current_window_bounds(&body)?;
        SettingsRepository::set_dock_y(db, current.y + current.height / 2.0 - HANDLE_HEIGHT / 2.0)?;
        follow_chrome_y_to_body(app).ok();
        return Ok(());
    }

    let home = SettingsRepository::get_home_shape(db)?;
    if home == HomeShape::Ball {
        capture_temp_body(app)?;
        persist_body_size_only(app)?;
    } else {
        persist_body_bounds(app)?;
    }
    if let Some(edge) = detect_nearest_edge(&body) {
        let bounds = current_window_bounds(&body)?;
        dock_at(
            app,
            edge,
            bounds.y + bounds.height / 2.0 - HANDLE_HEIGHT / 2.0,
        )?;
    }
    Ok(())
}

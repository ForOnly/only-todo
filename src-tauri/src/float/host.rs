use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager};

use crate::domain::{
    derive_chrome, panel_visible, BodyView, ChromeKind, CompanionDragEndResult, CompanionPlacement,
    CompanionSession, CompanionSurface, CompanionVisibility, DockEdge, HomeShape, PanelMode,
    WindowBounds,
};
use crate::errors::AppError;
use crate::repository::settings_repository::SettingsRepository;
use crate::services::settings_service::SettingsService;
use crate::services::todo_service::TodoService;
use crate::state::AppState;

use super::geometry::{
    body_beside_handle, clamp_panel_in_work, closer_vertical_edge, current_window_bounds,
    detect_nearest_edge, dock_drag_outcome, dock_threshold, handle_bounds, os_primary_mouse_down,
    panel_from_ball, snap_ball_to_work_area, snap_handle_to_work_area, window_lost,
    work_area_logical, DockDragOutcome, BALL_SIZE, HANDLE_HEIGHT, PANEL_DEFAULT_H, PANEL_DEFAULT_W,
};
use super::surfaces::{
    apply_bounds, apply_position, body_window, chrome_window, hide_window, set_always_on_top,
};
use super::SESSION_EVENT;

struct ClusterState {
    chrome_inside: bool,
    body_inside: bool,
    body_focused: bool,
    preview_gen: u64,
    unpreview_gen: u64,
}

struct RuntimeState {
    visibility: CompanionVisibility,
    panel_mode: PanelMode,
    body_view: BodyView,
    /// 圆球家临时面板位置，不写入 companion.panel_bounds 的 x/y
    temp_body_bounds: Option<WindowBounds>,
}

pub struct FloatHost {
    cluster: Mutex<ClusterState>,
    runtime: Mutex<RuntimeState>,
    applying: AtomicBool,
    apply_gen: AtomicU64,
}

impl Default for FloatHost {
    fn default() -> Self {
        Self::new()
    }
}

impl FloatHost {
    pub fn new() -> Self {
        Self {
            cluster: Mutex::new(ClusterState {
                chrome_inside: false,
                body_inside: false,
                body_focused: false,
                preview_gen: 0,
                unpreview_gen: 0,
            }),
            runtime: Mutex::new(RuntimeState {
                visibility: CompanionVisibility::Hidden,
                panel_mode: PanelMode::Closed,
                body_view: BodyView::TodoMini,
                temp_body_bounds: None,
            }),
            applying: AtomicBool::new(false),
            apply_gen: AtomicU64::new(0),
        }
    }

    pub fn is_applying(&self) -> bool {
        self.applying.load(Ordering::SeqCst)
    }

    pub fn show(app: &AppHandle) -> Result<CompanionSession, AppError> {
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
        {
            let state = app_state(app)?;
            let db = &state.db;
            let home = SettingsRepository::get_home_shape(db)?;
            let docked = SettingsRepository::get_placement(db)? == CompanionPlacement::Docked;
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
                    PanelMode::Closed | PanelMode::Preview => PanelMode::Pinned,
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
        set_always_on_top(app, always);
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
            apply_home_shape_change(app)?;
            need_apply = true;
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
            if let Some(body) = body_window(app) {
                if body.is_visible().unwrap_or(false) {
                    persist_body_size_only(app)?;
                }
            }
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

fn host_state(app: &AppHandle) -> Result<tauri::State<'_, FloatHost>, AppError> {
    app.try_state::<FloatHost>()
        .ok_or_else(|| AppError::InternalError {
            message: "FloatHost not managed".into(),
        })
}

fn lock_runtime(host: &FloatHost) -> Result<std::sync::MutexGuard<'_, RuntimeState>, AppError> {
    host.runtime.lock().map_err(|_| AppError::InternalError {
        message: "float host lock poisoned".into(),
    })
}

fn app_state(app: &AppHandle) -> Result<tauri::State<'_, AppState>, AppError> {
    app.try_state::<AppState>()
        .ok_or_else(|| AppError::InternalError {
            message: "AppState not managed".into(),
        })
}

fn companion_is_shown(app: &AppHandle) -> bool {
    chrome_window(app)
        .map(|window| window.is_visible().unwrap_or(false))
        .unwrap_or(false)
        || body_window(app)
            .map(|window| window.is_visible().unwrap_or(false))
            .unwrap_or(false)
}

fn companion_lost(app: &AppHandle) -> bool {
    let chrome_vis = chrome_window(app)
        .map(|window| window.is_visible().unwrap_or(false))
        .unwrap_or(false);
    let body_vis = body_window(app)
        .map(|window| window.is_visible().unwrap_or(false))
        .unwrap_or(false);
    if !chrome_vis && !body_vis {
        return true;
    }
    if chrome_vis {
        if let Some(window) = chrome_window(app) {
            if window_lost(&window) {
                return true;
            }
        }
    }
    if body_vis {
        if let Some(window) = body_window(app) {
            if window_lost(&window) {
                return true;
            }
        }
    }
    false
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
    if let Some(saved) = bounds {
        SettingsRepository::set_panel_bounds(db, &saved)?;
    }
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
            panel.x = drop.x;
            panel.y = drop.y;
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
        let edge = SettingsRepository::get_dock_edge(db)?;
        let current = current_window_bounds(&body)?;
        let work = work_area_logical(&body)?;
        let chrome_w = chrome_window(app)
            .and_then(|window| current_window_bounds(&window).ok())
            .map(|b| b.width)
            .unwrap_or(HANDLE_HEIGHT);
        let threshold = dock_threshold(chrome_w);
        persist_body_size_only(app)?;
        match dock_drag_outcome(&current, work, edge, threshold) {
            DockDragOutcome::StayCurrent => {
                SettingsRepository::set_dock_y(
                    db,
                    current.y + current.height / 2.0 - HANDLE_HEIGHT / 2.0,
                )?;
            }
            DockDragOutcome::Switch(next) => {
                SettingsRepository::set_dock_edge(db, next)?;
                SettingsRepository::set_dock_y(
                    db,
                    current.y + current.height / 2.0 - HANDLE_HEIGHT / 2.0,
                )?;
            }
            DockDragOutcome::Undock => undock_to_home(app, &current)?,
        }
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

fn free_body_bounds(
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

fn persist_panel_size(
    db: &crate::infrastructure::database::Database,
    width: f64,
    height: f64,
) -> Result<(), AppError> {
    let mut saved = SettingsRepository::get_panel_bounds(db)?;
    saved.width = width;
    saved.height = height;
    SettingsRepository::set_panel_bounds(db, &saved)
}

fn capture_temp_body(app: &AppHandle) -> Result<(), AppError> {
    let Some(body) = body_window(app) else {
        return Ok(());
    };
    if !body.is_visible().unwrap_or(false) {
        return Ok(());
    }
    let bounds = current_window_bounds(&body)?;
    let host = host_state(app)?;
    lock_runtime(&host)?.temp_body_bounds = Some(bounds);
    Ok(())
}

fn apply_home_shape_change(app: &AppHandle) -> Result<(), AppError> {
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

fn persist_body_bounds(app: &AppHandle) -> Result<(), AppError> {
    let Some(body) = body_window(app) else {
        return Ok(());
    };
    if !body.is_visible().unwrap_or(false) {
        return Ok(());
    }
    let bounds = current_window_bounds(&body)?;
    SettingsRepository::set_panel_bounds(&app_state(app)?.db, &bounds)
}

fn persist_body_size_only(app: &AppHandle) -> Result<(), AppError> {
    let Some(body) = body_window(app) else {
        return Ok(());
    };
    if !body.is_visible().unwrap_or(false) {
        return Ok(());
    }
    let current = current_window_bounds(&body)?;
    let db = &app_state(app)?.db;
    let mut saved = SettingsRepository::get_panel_bounds(db)?;
    saved.width = current.width;
    saved.height = current.height;
    SettingsRepository::set_panel_bounds(db, &saved)
}

fn persist_chrome_ball(app: &AppHandle) -> Result<(), AppError> {
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

fn apply_and_emit(app: &AppHandle) -> Result<CompanionSession, AppError> {
    apply_surfaces(app)?;
    let session = snapshot(app)?;
    let _ = app.emit(SESSION_EVENT, &session);
    super::update_tray_tooltip(app);
    Ok(session)
}

fn apply_surfaces(app: &AppHandle) -> Result<(), AppError> {
    let host = host_state(app)?;
    host.applying.store(true, Ordering::SeqCst);
    let gen = host.apply_gen.fetch_add(1, Ordering::SeqCst) + 1;
    let result = apply_surfaces_inner(app);
    let app_clone = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(80));
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
    set_always_on_top(app, always_on_top);

    let (visibility, panel_mode) = {
        let host = host_state(app)?;
        let rt = lock_runtime(&host)?;
        (rt.visibility, rt.panel_mode)
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
        return Ok(());
    }

    // 自由/临时面板：先藏 chrome 再出 body；贴边钉住时条留下
    if body_on {
        match placement {
            CompanionPlacement::Free => {
                hide_window(&chrome)?;
                let mut bounds = free_body_bounds(app, db, home, &body, &chrome)?;
                if let Ok(work) = work_area_logical(&body).or_else(|_| work_area_logical(&chrome)) {
                    clamp_panel_in_work(work, &mut bounds);
                }
                apply_bounds(&body, &bounds, true)?;
                if panel_mode == PanelMode::Pinned {
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
                if panel_mode == PanelMode::Pinned {
                    let _ = body.set_focus();
                }
            }
        }
        return Ok(());
    }

    hide_window(&body)?;
    match chrome_kind {
        ChromeKind::Hidden => hide_window(&chrome)?,
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
        }
    }
    Ok(())
}

pub fn snapshot(app: &AppHandle) -> Result<CompanionSession, AppError> {
    let state = app_state(app)?;
    let db = &state.db;
    let (visibility, panel_mode, body_view) = {
        let host = host_state(app)?;
        let rt = lock_runtime(&host)?;
        (rt.visibility, rt.panel_mode, rt.body_view)
    };
    let placement = SettingsRepository::get_placement(db)?;
    let home_shape = SettingsRepository::get_home_shape(db)?;
    let panel_mode = if placement == CompanionPlacement::Free
        && home_shape == HomeShape::Panel
        && visibility == CompanionVisibility::Shown
    {
        PanelMode::Pinned
    } else {
        panel_mode
    };
    let chrome = derive_chrome(visibility, placement, home_shape, panel_mode);
    let active_count = TodoService::count_active(db)?;
    let (overdue_count, due_today_count) = TodoService::count_due_urgency(db)?;
    Ok(CompanionSession {
        visibility,
        placement,
        home_shape,
        panel_mode,
        chrome,
        dock_edge: SettingsRepository::get_dock_edge(db)?,
        dock_y: SettingsRepository::get_dock_y(db)?,
        body_view,
        hover_preview: SettingsRepository::get_hover_preview(db)?,
        active_count,
        overdue_count,
        due_today_count,
    })
}

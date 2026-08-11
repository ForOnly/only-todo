use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use tauri::{AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, State, WebviewWindow};

use crate::domain::{DockEdge, FloatDisplayMode, FloatWindowState, WindowBounds};
use crate::errors::AppError;
use crate::repository::settings_repository::SettingsRepository;
use crate::services::settings_service::SettingsService;
use crate::services::todo_service::TodoService;
use crate::state::AppState;

pub const FLOAT_LABEL: &str = "float";

/// 程序化改尺寸期间禁止 Moved/Resized 写回 bounds（世代解锁，避免重叠操作误提前开锁）
pub struct FloatGeometryLock {
    pub locked: AtomicBool,
    pub generation: AtomicU64,
}

/// 挂边 HWND 最小宽度，避免 WebView2 不绘制 / Win 最小宽度把窗顶出屏幕（视觉蓝条仍为 8px）
const DOCK_HWND_THICKNESS: f64 = 48.0;
const EDGE_THRESHOLD: f64 = 24.0;
const PANEL_DEFAULT_W: f64 = 320.0;
const PANEL_DEFAULT_H: f64 = 480.0;
const BALL_WINDOW_SIZE: f64 = 64.0;

#[tauri::command]
pub fn hide_to_tray(app: AppHandle) -> Result<(), AppError> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|error| AppError::InternalError {
            message: error.to_string(),
        })?;
    }
    Ok(())
}

#[tauri::command]
pub fn show_main_window(app: AppHandle, todo_id: Option<String>) -> Result<(), AppError> {
    show_main_window_impl(app, todo_id)
}

#[tauri::command]
pub fn show_floating_window(app: AppHandle) -> Result<(), AppError> {
    show_floating_window_impl(app)
}

#[tauri::command]
pub fn hide_floating_window(app: AppHandle) -> Result<(), AppError> {
    hide_floating_window_impl(app)
}

#[tauri::command]
pub fn toggle_floating_window(app: AppHandle) -> Result<(), AppError> {
    toggle_floating_window_impl(app)
}

#[tauri::command]
pub fn get_active_todo_count(state: State<AppState>) -> Result<u64, AppError> {
    TodoService::count_active(&state.db)
}

#[tauri::command]
pub fn get_float_window_state(state: State<AppState>) -> Result<FloatWindowState, AppError> {
    build_float_window_state(&state.db)
}

#[tauri::command]
pub fn set_float_display_mode(
    app: AppHandle,
    state: State<AppState>,
    mode: FloatDisplayMode,
) -> Result<FloatWindowState, AppError> {
    set_float_display_mode_impl(&app, &state.db, mode)
}

#[tauri::command]
pub fn dock_float_window(
    app: AppHandle,
    state: State<AppState>,
    edge: Option<DockEdge>,
) -> Result<FloatWindowState, AppError> {
    dock_float_window_impl(&app, &state.db, edge)
}

#[tauri::command]
pub fn undock_float_window(
    app: AppHandle,
    state: State<AppState>,
) -> Result<FloatWindowState, AppError> {
    let home = SettingsRepository::get_home_mode(&state.db)?;
    set_float_display_mode_impl(&app, &state.db, home)
}

#[tauri::command]
pub fn try_dock_on_edge(
    app: AppHandle,
    state: State<AppState>,
) -> Result<FloatWindowState, AppError> {
    try_dock_on_edge_impl(&app, &state.db)
}

/// 挂边态悬停：临时展开为 panel 尺寸（不改 display_mode）
/// client_y：相对 float 窗口客户区的 Y；Rust 转为屏幕逻辑坐标
#[tauri::command]
pub fn peek_docked_float(
    app: AppHandle,
    state: State<AppState>,
    client_y: Option<f64>,
) -> Result<FloatWindowState, AppError> {
    peek_docked_float_impl(&app, &state.db, client_y)
}

/// 挂边态移开：收回竖线
#[tauri::command]
pub fn unpeek_docked_float(
    app: AppHandle,
    state: State<AppState>,
) -> Result<FloatWindowState, AppError> {
    unpeek_docked_float_impl(&app, &state.db)
}

/// OS 级主按键是否仍按下（native drag 后 WebView pointerup 常丢失）
#[tauri::command]
pub fn is_primary_mouse_down() -> bool {
    os_primary_mouse_down()
}

fn os_primary_mouse_down() -> bool {
    #[cfg(windows)]
    {
        // GetAsyncKeyState 最高位表示当前按下
        const VK_LBUTTON: i32 = 0x01;
        unsafe {
            (windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(VK_LBUTTON) as u16)
                & 0x8000
                != 0
        }
    }
    #[cfg(not(windows))]
    {
        false
    }
}

/// Tauri Command 与通知点击共用的主窗口显示逻辑。
pub fn show_main_window_impl(app: AppHandle, todo_id: Option<String>) -> Result<(), AppError> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|error| AppError::InternalError {
            message: error.to_string(),
        })?;
        window.unminimize().ok();
        window.set_focus().ok();
    }

    if let Some(id) = todo_id {
        app.emit("navigate-to-todo", id)
            .map_err(|error| AppError::InternalError {
                message: error.to_string(),
            })?;
    }

    Ok(())
}

pub fn show_floating_window_impl(app: AppHandle) -> Result<(), AppError> {
    let Some(window) = app.get_webview_window(FLOAT_LABEL) else {
        return Ok(());
    };

    if let Some(state) = app.try_state::<AppState>() {
        apply_float_mode_from_settings(&app, &window, &state.db)?;
        let always_on_top = SettingsService::float_always_on_top(&state.db)?;
        window.set_always_on_top(always_on_top).ok();
    }

    window.show().map_err(|error| AppError::InternalError {
        message: error.to_string(),
    })?;
    window.set_focus().ok();
    Ok(())
}

pub fn hide_floating_window_impl(app: AppHandle) -> Result<(), AppError> {
    if let Some(window) = app.get_webview_window(FLOAT_LABEL) {
        window.hide().map_err(|error| AppError::InternalError {
            message: error.to_string(),
        })?;
    }
    Ok(())
}

pub fn toggle_floating_window_impl(app: AppHandle) -> Result<(), AppError> {
    if let Some(window) = app.get_webview_window(FLOAT_LABEL) {
        // 可见但过小/飞出工作区：视为丢失，走 show+修复而不是 hide
        if window.is_visible().unwrap_or(false) && !float_window_lost(&window) {
            return hide_floating_window_impl(app);
        }
    }
    show_floating_window_impl(app)
}

pub fn apply_float_bounds(window: &WebviewWindow, bounds: &WindowBounds) -> Result<(), AppError> {
    let _ = window.set_shadow(false);
    window
        .set_size(LogicalSize::new(bounds.width, bounds.height))
        .map_err(|error| AppError::InternalError {
            message: error.to_string(),
        })?;
    window
        .set_position(LogicalPosition::new(bounds.x, bounds.y))
        .map_err(|error| AppError::InternalError {
            message: error.to_string(),
        })?;
    let _ = window.show();
    Ok(())
}

fn float_window_lost(window: &WebviewWindow) -> bool {
    let Ok(bounds) = current_window_bounds(window) else {
        return true;
    };
    if bounds.width < 40.0 || bounds.height < 40.0 {
        return true;
    }
    let Ok((work_x, work_y, work_w, work_h)) = work_area_logical(window) else {
        return true;
    };
    let cx = bounds.x + bounds.width / 2.0;
    let cy = bounds.y + bounds.height / 2.0;
    cx < work_x || cx > work_x + work_w || cy < work_y || cy > work_y + work_h
}

fn with_geometry_lock(app: &AppHandle, f: impl FnOnce() -> Result<(), AppError>) -> Result<(), AppError> {
    let lock = app.try_state::<FloatGeometryLock>();
    let gen = if let Some(lock) = &lock {
        lock.locked.store(true, Ordering::SeqCst);
        lock.generation.fetch_add(1, Ordering::SeqCst) + 1
    } else {
        0
    };
    let result = f();
    // 短暂延迟解锁，避免 Moved/Resized 晚到仍写回中间态 bounds；仅本代可解锁
    if lock.is_some() {
        let app = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(80));
            if let Some(lock) = app.try_state::<FloatGeometryLock>() {
                if lock.generation.load(Ordering::SeqCst) == gen {
                    lock.locked.store(false, Ordering::SeqCst);
                }
            }
        });
    }
    result
}

pub fn is_geometry_locked(app: &AppHandle) -> bool {
    app.try_state::<FloatGeometryLock>()
        .map(|lock| lock.locked.load(Ordering::SeqCst))
        .unwrap_or(false)
}

pub fn apply_float_mode_from_settings(
    app: &AppHandle,
    window: &WebviewWindow,
    db: &crate::infrastructure::database::Database,
) -> Result<(), AppError> {
    let display = SettingsRepository::get_display_mode(db)?;
    // 托盘再显示：挂边保持；否则回到主形态（临时 panel 不跨隐藏记住）
    let mode = if display == FloatDisplayMode::Docked {
        FloatDisplayMode::Docked
    } else {
        SettingsRepository::get_home_mode(db)?
    };
    if display != mode {
        SettingsRepository::set_display_mode(db, mode)?;
    }
    with_geometry_lock(app, || apply_mode_geometry(window, db, mode))
}

fn apply_mode_geometry(
    window: &WebviewWindow,
    db: &crate::infrastructure::database::Database,
    mode: FloatDisplayMode,
) -> Result<(), AppError> {
    match mode {
        FloatDisplayMode::Ball => {
            let mut bounds = SettingsRepository::get_ball_bounds(db)?;
            normalize_ball_bounds(&mut bounds);
            apply_float_bounds(window, &bounds)
        }
        FloatDisplayMode::Panel => {
            let bounds = SettingsRepository::get_panel_bounds(db)?;
            apply_float_bounds(window, &bounds)
        }
        FloatDisplayMode::Docked => {
            let edge = SettingsRepository::get_dock_edge(db)?;
            apply_docked_bounds(window, db, edge)
        }
    }
}

/// 左右贴边：HWND 48×64（与圆球同高），视觉 8px 蓝条由前端绘制
pub fn apply_docked_bounds(
    window: &WebviewWindow,
    db: &crate::infrastructure::database::Database,
    edge: DockEdge,
) -> Result<(), AppError> {
    let (work_x, work_y, work_w, work_h) = work_area_logical(window)?;
    let edge = normalize_vertical_edge(edge);
    let height = BALL_WINDOW_SIZE;
    let width = DOCK_HWND_THICKNESS;
    let max_y = (work_y + work_h - height).max(work_y);

    let preferred_y = current_window_bounds(window)
        .ok()
        .map(|bounds| {
            if (bounds.height - BALL_WINDOW_SIZE).abs() < 8.0 {
                bounds.y
            } else {
                bounds.y + (bounds.height - height) / 2.0
            }
        })
        .or_else(|| {
            SettingsRepository::get_ball_bounds(db)
                .ok()
                .map(|bounds| bounds.y)
        })
        .unwrap_or(work_y);
    let y = preferred_y.clamp(work_y, max_y);

    let (x, y, width, height) = match edge {
        DockEdge::Left => (work_x, y, width, height),
        DockEdge::Right => (work_x + work_w - width, y, width, height),
        DockEdge::Top => (work_x, work_y, work_w, width),
        DockEdge::Bottom => (work_x, work_y + work_h - width, work_w, width),
    };

    apply_float_bounds(
        window,
        &WindowBounds {
            x,
            y,
            width,
            height,
        },
    )
}

/// 挂边悬停展开：贴同侧展开为 panel 尺寸；client_y 转为逻辑屏幕 Y 后纵向对齐
fn apply_peeked_bounds(
    window: &WebviewWindow,
    db: &crate::infrastructure::database::Database,
    edge: DockEdge,
    client_y: Option<f64>,
) -> Result<(), AppError> {
    let (work_x, work_y, work_w, work_h) = work_area_logical(window)?;
    let edge = normalize_vertical_edge(edge);
    let panel = SettingsRepository::get_panel_bounds(db).unwrap_or_else(|_| WindowBounds {
        x: 100.0,
        y: 100.0,
        width: PANEL_DEFAULT_W,
        height: PANEL_DEFAULT_H,
    });

    let width = panel.width.clamp(200.0, work_w);
    let height = panel.height.clamp(200.0, work_h);
    let max_y = (work_y + work_h - height).max(work_y);

    let cursor_screen_y = match client_y {
        Some(cy) => {
            let scale = window.scale_factor().map_err(|error| AppError::InternalError {
                message: error.to_string(),
            })?;
            let pos = window
                .inner_position()
                .map_err(|error| AppError::InternalError {
                    message: error.to_string(),
                })?;
            Some(pos.y as f64 / scale + cy)
        }
        None => None,
    };

    let y = match cursor_screen_y {
        Some(cy) => (cy - height / 2.0).clamp(work_y, max_y),
        None => (work_y + (work_h - height) / 2.0).clamp(work_y, max_y),
    };

    let x = match edge {
        DockEdge::Left => work_x,
        DockEdge::Right => work_x + work_w - width,
        DockEdge::Top => work_x + ((work_w - width) / 2.0).max(0.0),
        DockEdge::Bottom => work_x + ((work_w - width) / 2.0).max(0.0),
    };

    let y = match edge {
        DockEdge::Top => work_y,
        DockEdge::Bottom => work_y + work_h - height,
        _ => y,
    };

    apply_float_bounds(
        window,
        &WindowBounds {
            x,
            y,
            width,
            height,
        },
    )
}

fn normalize_ball_bounds(bounds: &mut WindowBounds) {
    bounds.width = BALL_WINDOW_SIZE;
    bounds.height = BALL_WINDOW_SIZE;
}

fn normalize_vertical_edge(edge: DockEdge) -> DockEdge {
    match edge {
        DockEdge::Left | DockEdge::Right => edge,
        DockEdge::Top | DockEdge::Bottom => DockEdge::Right,
    }
}

fn work_area_logical(window: &WebviewWindow) -> Result<(f64, f64, f64, f64), AppError> {
    let monitor = window
        .current_monitor()
        .map_err(|error| AppError::InternalError {
            message: error.to_string(),
        })?
        .ok_or_else(|| AppError::InternalError {
            message: "no monitor for float window".into(),
        })?;

    let scale = monitor.scale_factor();
    let work = monitor.work_area();
    Ok((
        work.position.x as f64 / scale,
        work.position.y as f64 / scale,
        work.size.width as f64 / scale,
        work.size.height as f64 / scale,
    ))
}

/// 仅检测左右边（竖线挂边）
fn detect_nearest_edge(window: &WebviewWindow) -> Option<DockEdge> {
    let position = window.outer_position().ok()?;
    let size = window.outer_size().ok()?;
    let scale = window.scale_factor().ok()?;
    let x = position.x as f64 / scale;
    let w = size.width as f64 / scale;

    let (work_x, _work_y, work_w, _work_h) = work_area_logical(window).ok()?;

    let dist_left = (x - work_x).abs();
    let dist_right = (work_x + work_w - (x + w)).abs();

    let min = dist_left.min(dist_right);
    if min > EDGE_THRESHOLD {
        return None;
    }

    if dist_left <= dist_right {
        Some(DockEdge::Left)
    } else {
        Some(DockEdge::Right)
    }
}

fn set_float_display_mode_impl(
    app: &AppHandle,
    db: &crate::infrastructure::database::Database,
    mode: FloatDisplayMode,
) -> Result<FloatWindowState, AppError> {
    let Some(window) = app.get_webview_window(FLOAT_LABEL) else {
        return build_float_window_state(db);
    };

    let current_mode = SettingsRepository::get_display_mode(db)?;

    if current_mode != mode && current_mode != FloatDisplayMode::Docked {
        persist_current_bounds(app, &window, current_mode)?;
    }

    // 先写 mode，再改几何，避免 Moved/Resized 按旧形态写回；失败则回滚 mode
    SettingsRepository::set_display_mode(db, mode)?;

    if let Err(error) = with_geometry_lock(app, || apply_mode_geometry(&window, db, mode)) {
        SettingsRepository::set_display_mode(db, current_mode)?;
        let _ = with_geometry_lock(app, || apply_mode_geometry(&window, db, current_mode));
        return Err(error);
    }

    build_float_window_state(db)
}

fn dock_float_window_impl(
    app: &AppHandle,
    db: &crate::infrastructure::database::Database,
    edge: Option<DockEdge>,
) -> Result<FloatWindowState, AppError> {
    let Some(window) = app.get_webview_window(FLOAT_LABEL) else {
        return build_float_window_state(db);
    };

    let current_mode = SettingsRepository::get_display_mode(db)?;
    if current_mode != FloatDisplayMode::Docked {
        persist_current_bounds(app, &window, current_mode)?;
    }

    let edge = normalize_vertical_edge(
        edge.unwrap_or_else(|| detect_nearest_edge(&window).unwrap_or(DockEdge::Right)),
    );

    SettingsRepository::set_dock_edge(db, edge)?;
    SettingsRepository::set_display_mode(db, FloatDisplayMode::Docked)?;

    if let Err(error) = with_geometry_lock(app, || apply_docked_bounds(&window, db, edge)) {
        SettingsRepository::set_display_mode(db, current_mode)?;
        let _ = with_geometry_lock(app, || apply_mode_geometry(&window, db, current_mode));
        return Err(error);
    }

    build_float_window_state(db)
}

fn try_dock_on_edge_impl(
    app: &AppHandle,
    db: &crate::infrastructure::database::Database,
) -> Result<FloatWindowState, AppError> {
    let Some(window) = app.get_webview_window(FLOAT_LABEL) else {
        return build_float_window_state(db);
    };

    let mode = SettingsRepository::get_display_mode(db)?;

    if mode == FloatDisplayMode::Docked {
        let edge = SettingsRepository::get_dock_edge(db)?;
        if !is_near_work_edge(&window, edge) {
            let home = SettingsRepository::get_home_mode(db)?;
            let current = current_window_bounds(&window)?;
            match home {
                FloatDisplayMode::Panel => {
                    let mut panel =
                        SettingsRepository::get_panel_bounds(db).unwrap_or(WindowBounds {
                            x: 100.0,
                            y: 100.0,
                            width: PANEL_DEFAULT_W,
                            height: PANEL_DEFAULT_H,
                        });
                    panel.x = current.x;
                    panel.y = current.y;
                    if current.width > DOCK_HWND_THICKNESS * 2.0 {
                        panel.width = current.width;
                        panel.height = current.height;
                    }
                    SettingsRepository::set_panel_bounds(db, &panel)?;
                }
                FloatDisplayMode::Ball | FloatDisplayMode::Docked => {
                    let mut ball = SettingsRepository::get_ball_bounds(db).unwrap_or(WindowBounds {
                        x: current.x,
                        y: current.y,
                        width: BALL_WINDOW_SIZE,
                        height: BALL_WINDOW_SIZE,
                    });
                    ball.x = current.x;
                    ball.y = current.y;
                    normalize_ball_bounds(&mut ball);
                    SettingsRepository::set_ball_bounds(db, &ball)?;
                }
            }
            return set_float_display_mode_impl(app, db, home);
        }
        // 拖离后未出边：收回竖线
        with_geometry_lock(app, || apply_docked_bounds(&window, db, edge))?;
        return build_float_window_state(db);
    }

    if let Some(edge) = detect_nearest_edge(&window) {
        return dock_float_window_impl(app, db, Some(edge));
    }

    persist_current_bounds(app, &window, mode)?;
    build_float_window_state(db)
}

fn peek_docked_float_impl(
    app: &AppHandle,
    db: &crate::infrastructure::database::Database,
    client_y: Option<f64>,
) -> Result<FloatWindowState, AppError> {
    let Some(window) = app.get_webview_window(FLOAT_LABEL) else {
        return build_float_window_state(db);
    };

    let mode = SettingsRepository::get_display_mode(db)?;
    if mode != FloatDisplayMode::Docked {
        return build_float_window_state(db);
    }

    let edge = SettingsRepository::get_dock_edge(db)?;
    with_geometry_lock(app, || apply_peeked_bounds(&window, db, edge, client_y))?;
    build_float_window_state(db)
}

fn unpeek_docked_float_impl(
    app: &AppHandle,
    db: &crate::infrastructure::database::Database,
) -> Result<FloatWindowState, AppError> {
    let Some(window) = app.get_webview_window(FLOAT_LABEL) else {
        return build_float_window_state(db);
    };

    let mode = SettingsRepository::get_display_mode(db)?;
    if mode != FloatDisplayMode::Docked {
        return build_float_window_state(db);
    }

    let edge = SettingsRepository::get_dock_edge(db)?;
    with_geometry_lock(app, || apply_docked_bounds(&window, db, edge))?;
    build_float_window_state(db)
}

fn is_near_work_edge(window: &WebviewWindow, edge: DockEdge) -> bool {
    let Ok(bounds) = current_window_bounds(window) else {
        return false;
    };
    let Ok((work_x, work_y, work_w, work_h)) = work_area_logical(window) else {
        return false;
    };

    let edge = normalize_vertical_edge(edge);
    match edge {
        DockEdge::Left => (bounds.x - work_x).abs() <= EDGE_THRESHOLD,
        DockEdge::Right => (work_x + work_w - (bounds.x + bounds.width)).abs() <= EDGE_THRESHOLD,
        DockEdge::Top => (bounds.y - work_y).abs() <= EDGE_THRESHOLD,
        DockEdge::Bottom => (work_y + work_h - (bounds.y + bounds.height)).abs() <= EDGE_THRESHOLD,
    }
}

fn current_window_bounds(window: &WebviewWindow) -> Result<WindowBounds, AppError> {
    let scale = window.scale_factor().map_err(|error| AppError::InternalError {
        message: error.to_string(),
    })?;
    let position = window
        .outer_position()
        .map_err(|error| AppError::InternalError {
            message: error.to_string(),
        })?;
    let size = window
        .outer_size()
        .map_err(|error| AppError::InternalError {
            message: error.to_string(),
        })?;

    Ok(WindowBounds {
        x: position.x as f64 / scale,
        y: position.y as f64 / scale,
        width: size.width as f64 / scale,
        height: size.height as f64 / scale,
    })
}

fn persist_current_bounds(
    app: &AppHandle,
    window: &WebviewWindow,
    mode: FloatDisplayMode,
) -> Result<(), AppError> {
    let Some(state) = app.try_state::<AppState>() else {
        return Ok(());
    };

    let mut bounds = current_window_bounds(window)?;
    match mode {
        FloatDisplayMode::Ball => {
            normalize_ball_bounds(&mut bounds);
            SettingsRepository::set_ball_bounds(&state.db, &bounds)
        }
        FloatDisplayMode::Panel => SettingsRepository::set_panel_bounds(&state.db, &bounds),
        FloatDisplayMode::Docked => Ok(()),
    }
}

pub fn persist_float_bounds(app: &AppHandle) -> Result<(), AppError> {
    if is_geometry_locked(app) {
        return Ok(());
    }

    let Some(window) = app.get_webview_window(FLOAT_LABEL) else {
        return Ok(());
    };
    let Some(state) = app.try_state::<AppState>() else {
        return Ok(());
    };

    let mode = SettingsRepository::get_display_mode(&state.db)?;
    if mode == FloatDisplayMode::Docked {
        return Ok(());
    }

    persist_current_bounds(app, &window, mode)
}

pub fn update_tray_tooltip(app: &AppHandle) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let count = TodoService::count_active(&state.db).unwrap_or(0);
    let tooltip = format!("Only Todo ({count})");
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_tooltip(Some(tooltip));
    }
}

fn build_float_window_state(
    db: &crate::infrastructure::database::Database,
) -> Result<FloatWindowState, AppError> {
    let display_mode = SettingsRepository::get_display_mode(db)?;
    let dock_edge = SettingsRepository::get_dock_edge(db)?;
    let active_count = TodoService::count_active(db)?;
    let (overdue_count, due_today_count) = TodoService::count_due_urgency(db)?;

    let default_mode = SettingsRepository::get_home_mode(db)?;

    Ok(FloatWindowState {
        display_mode,
        default_mode,
        dock_edge,
        active_count,
        overdue_count,
        due_today_count,
    })
}

#[tauri::command]
pub fn health_check() -> Result<String, AppError> {
    Ok("ok".into())
}

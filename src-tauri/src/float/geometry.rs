use tauri::WebviewWindow;

use crate::domain::{DockEdge, WindowBounds};
use crate::errors::AppError;

pub const BALL_SIZE: f64 = 64.0;
pub const HANDLE_WIDTH: f64 = 16.0;
pub const HANDLE_HEIGHT: f64 = 48.0;
pub const EDGE_THRESHOLD: f64 = 24.0;
pub const PANEL_DEFAULT_W: f64 = 320.0;
pub const PANEL_DEFAULT_H: f64 = 480.0;
pub const PANEL_MIN: f64 = 200.0;

pub fn work_area_logical(window: &WebviewWindow) -> Result<(f64, f64, f64, f64), AppError> {
    let monitor = window
        .current_monitor()
        .map_err(|error| AppError::InternalError {
            message: error.to_string(),
        })?
        .ok_or_else(|| AppError::InternalError {
            message: "no monitor for companion window".into(),
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

pub fn inner_window_size(window: &WebviewWindow) -> Result<(f64, f64), AppError> {
    let scale = window
        .scale_factor()
        .map_err(|error| AppError::InternalError {
            message: error.to_string(),
        })?;
    let size = window
        .inner_size()
        .map_err(|error| AppError::InternalError {
            message: error.to_string(),
        })?;
    Ok((size.width as f64 / scale, size.height as f64 / scale))
}

pub fn current_window_bounds(window: &WebviewWindow) -> Result<WindowBounds, AppError> {
    let scale = window
        .scale_factor()
        .map_err(|error| AppError::InternalError {
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

pub fn clamp_panel_size(bounds: &mut WindowBounds, work_w: f64, work_h: f64) {
    bounds.width = bounds.width.clamp(PANEL_MIN, work_w.max(PANEL_MIN));
    bounds.height = bounds.height.clamp(PANEL_MIN, work_h.max(PANEL_MIN));
}

pub fn clamp_panel_in_work(work: (f64, f64, f64, f64), bounds: &mut WindowBounds) {
    let (work_x, work_y, work_w, work_h) = work;
    clamp_panel_size(bounds, work_w, work_h);
    let max_x = (work_x + work_w - bounds.width).max(work_x);
    let max_y = (work_y + work_h - bounds.height).max(work_y);
    bounds.x = bounds.x.clamp(work_x, max_x);
    bounds.y = bounds.y.clamp(work_y, max_y);
}

pub fn handle_bounds(work: (f64, f64, f64, f64), edge: DockEdge, y: f64) -> WindowBounds {
    let (work_x, work_y, work_w, work_h) = work;
    let max_y = (work_y + work_h - HANDLE_HEIGHT).max(work_y);
    let y = y.clamp(work_y, max_y);
    let x = match edge {
        DockEdge::Left => work_x,
        DockEdge::Right => work_x + work_w - HANDLE_WIDTH,
    };
    WindowBounds {
        x,
        y,
        width: HANDLE_WIDTH,
        height: HANDLE_HEIGHT,
    }
}

/// WebView2 可能抬高 HWND；按实际外框把条完整收进工作区
pub fn snap_handle_to_work_area(
    work: (f64, f64, f64, f64),
    edge: DockEdge,
    requested_y: f64,
    actual: &WindowBounds,
) -> WindowBounds {
    let (work_x, work_y, work_w, work_h) = work;
    let width = actual.width.max(HANDLE_WIDTH);
    let height = actual.height.max(HANDLE_HEIGHT);
    let max_y = (work_y + work_h - height).max(work_y);
    let y = requested_y.clamp(work_y, max_y);
    let x = match edge {
        DockEdge::Left => work_x,
        DockEdge::Right => work_x + work_w - width,
    };
    WindowBounds {
        x,
        y,
        width,
        height,
    }
}

pub fn snap_ball_to_work_area(
    work: (f64, f64, f64, f64),
    x: f64,
    y: f64,
    actual: Option<&WindowBounds>,
) -> WindowBounds {
    let (work_x, work_y, work_w, work_h) = work;
    let width = actual.map(|b| b.width.max(BALL_SIZE)).unwrap_or(BALL_SIZE);
    let height = actual.map(|b| b.height.max(BALL_SIZE)).unwrap_or(BALL_SIZE);
    let max_x = (work_x + work_w - width).max(work_x);
    let max_y = (work_y + work_h - height).max(work_y);
    WindowBounds {
        x: x.clamp(work_x, max_x),
        y: y.clamp(work_y, max_y),
        width,
        height,
    }
}

pub fn body_beside_handle(
    work: (f64, f64, f64, f64),
    edge: DockEdge,
    handle: &WindowBounds,
    panel: &WindowBounds,
) -> WindowBounds {
    let (work_x, work_y, work_w, work_h) = work;
    let mut bounds = panel.clone();
    clamp_panel_size(&mut bounds, work_w, work_h);
    let max_y = (work_y + work_h - bounds.height).max(work_y);
    bounds.y = (handle.y + handle.height / 2.0 - bounds.height / 2.0).clamp(work_y, max_y);
    // 按视觉条宽让位，与加宽 HWND 重叠；透明区点击穿透后点到面板
    bounds.x = match edge {
        DockEdge::Left => work_x + HANDLE_WIDTH,
        DockEdge::Right => (work_x + work_w - HANDLE_WIDTH - bounds.width).max(work_x),
    };
    bounds
}

/// 从当前圆球外推面板：左半屏向右，右半屏向左；不用旧面板 x/y
pub fn panel_from_ball(
    work: (f64, f64, f64, f64),
    ball: &WindowBounds,
    panel: &WindowBounds,
) -> WindowBounds {
    let (work_x, work_y, work_w, work_h) = work;
    let mut bounds = panel.clone();
    clamp_panel_size(&mut bounds, work_w, work_h);
    let ball_cx = ball.x + ball.width / 2.0;
    let work_cx = work_x + work_w / 2.0;
    bounds.x = if ball_cx <= work_cx {
        ball.x + ball.width
    } else {
        ball.x - bounds.width
    };
    let max_x = (work_x + work_w - bounds.width).max(work_x);
    bounds.x = bounds.x.clamp(work_x, max_x);
    let max_y = (work_y + work_h - bounds.height).max(work_y);
    bounds.y = (ball.y + ball.height / 2.0 - bounds.height / 2.0).clamp(work_y, max_y);
    bounds
}

pub fn detect_nearest_edge(window: &WebviewWindow) -> Option<DockEdge> {
    let bounds = current_window_bounds(window).ok()?;
    let work = work_area_logical(window).ok()?;
    nearest_edge_of(&bounds, work, EDGE_THRESHOLD)
}

/// 贴边条视觉命中：外缘 16×48 + 内侧计数徽章
pub fn strip_visual_hit(bounds: &WindowBounds, edge: DockEdge, cx: f64, cy: f64) -> bool {
    let bar_h = HANDLE_HEIGHT.min(bounds.height);
    let y0 = bounds.y + ((bounds.height - bar_h) / 2.0).max(0.0);
    if cy < y0 || cy > y0 + bar_h {
        return false;
    }
    let badge = 20.0;
    match edge {
        DockEdge::Left => cx >= bounds.x && cx <= bounds.x + HANDLE_WIDTH + badge,
        DockEdge::Right => {
            cx >= bounds.x + bounds.width - HANDLE_WIDTH - badge && cx <= bounds.x + bounds.width
        }
    }
}

/// 圆球视觉命中：HWND 内切圆，四角穿透
pub fn ball_visual_hit(bounds: &WindowBounds, cx: f64, cy: f64) -> bool {
    let radius = (bounds.width.min(bounds.height) / 2.0 - 2.0).max(24.0);
    let mx = bounds.x + bounds.width / 2.0;
    let my = bounds.y + bounds.height / 2.0;
    (cx - mx).hypot(cy - my) <= radius
}

pub fn dock_threshold(chrome_width: f64) -> f64 {
    EDGE_THRESHOLD.max(chrome_width + 8.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockDragOutcome {
    StayCurrent,
    Switch(DockEdge),
    Undock,
}

/// 贴边拖拽结束：用被拖窗口的 outer，阈值按 chrome 实际宽度
pub fn dock_drag_outcome(
    bounds: &WindowBounds,
    work: (f64, f64, f64, f64),
    current: DockEdge,
    threshold: f64,
) -> DockDragOutcome {
    let (work_x, _work_y, work_w, _work_h) = work;
    let dist_left = (bounds.x - work_x).abs();
    let dist_right = (work_x + work_w - (bounds.x + bounds.width)).abs();
    let near_left = dist_left <= threshold;
    let near_right = dist_right <= threshold;
    match (near_left, near_right, current) {
        (true, false, DockEdge::Left) | (false, true, DockEdge::Right) => {
            DockDragOutcome::StayCurrent
        }
        (true, false, DockEdge::Right) => DockDragOutcome::Switch(DockEdge::Left),
        (false, true, DockEdge::Left) => DockDragOutcome::Switch(DockEdge::Right),
        (true, true, _) => {
            if dist_left <= dist_right {
                if current == DockEdge::Left {
                    DockDragOutcome::StayCurrent
                } else {
                    DockDragOutcome::Switch(DockEdge::Left)
                }
            } else if current == DockEdge::Right {
                DockDragOutcome::StayCurrent
            } else {
                DockDragOutcome::Switch(DockEdge::Right)
            }
        }
        _ => DockDragOutcome::Undock,
    }
}

fn nearest_edge_of(
    bounds: &WindowBounds,
    work: (f64, f64, f64, f64),
    threshold: f64,
) -> Option<DockEdge> {
    let (work_x, _work_y, work_w, _work_h) = work;
    let dist_left = (bounds.x - work_x).abs();
    let dist_right = (work_x + work_w - (bounds.x + bounds.width)).abs();
    let min = dist_left.min(dist_right);
    if min > threshold {
        return None;
    }
    if dist_left <= dist_right {
        Some(DockEdge::Left)
    } else {
        Some(DockEdge::Right)
    }
}

pub fn closer_vertical_edge(window: &WebviewWindow) -> DockEdge {
    let Ok(bounds) = current_window_bounds(window) else {
        return DockEdge::Right;
    };
    let Ok((work_x, _work_y, work_w, _work_h)) = work_area_logical(window) else {
        return DockEdge::Right;
    };
    let center = bounds.x + bounds.width / 2.0;
    if (center - work_x).abs() <= (work_x + work_w - center).abs() {
        DockEdge::Left
    } else {
        DockEdge::Right
    }
}

pub fn window_lost(window: &WebviewWindow) -> bool {
    let Ok(bounds) = current_window_bounds(window) else {
        return true;
    };
    if bounds.width < 8.0 || bounds.height < 8.0 {
        return true;
    }
    let Ok((work_x, work_y, work_w, work_h)) = work_area_logical(window) else {
        return true;
    };
    let cx = bounds.x + bounds.width / 2.0;
    let cy = bounds.y + bounds.height / 2.0;
    cx < work_x || cx > work_x + work_w || cy < work_y || cy > work_y + work_h
}

pub fn os_primary_mouse_down() -> bool {
    #[cfg(windows)]
    {
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

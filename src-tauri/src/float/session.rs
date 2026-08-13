//! 伴侣会话运行时状态（FloatHost）
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

use tauri::{AppHandle, Manager};

use crate::domain::{
    BodyView, ChromeKind, CompanionVisibility, DockEdge, PanelMode, WindowBounds,
};
use crate::errors::AppError;
use crate::state::AppState;

use super::geometry::window_lost;
use super::surfaces::{body_window, chrome_window};

pub(crate) struct ClusterState {
    pub(crate) chrome_inside: bool,
    pub(crate) body_inside: bool,
    pub(crate) body_focused: bool,
    pub(crate) preview_gen: u64,
    pub(crate) unpreview_gen: u64,
}

pub(crate) struct RuntimeState {
    pub(crate) visibility: CompanionVisibility,
    pub(crate) panel_mode: PanelMode,
    pub(crate) body_view: BodyView,
    /// 圆球家临时面板位置，不写入 companion.panel_bounds 的 x/y
    pub(crate) temp_body_bounds: Option<WindowBounds>,
}

pub struct FloatHost {
    pub(crate) cluster: Mutex<ClusterState>,
    pub(crate) runtime: Mutex<RuntimeState>,
    pub(crate) applying: AtomicBool,
    pub(crate) apply_gen: AtomicU64,
    /// 仅用户点开/钉住时聚焦面板
    pub(crate) focus_body_once: AtomicBool,
    pub(crate) last_aot: Mutex<Option<bool>>,
    pub(crate) chrome_ignore: AtomicBool,
    pub(crate) chrome_hit: Mutex<Option<(ChromeKind, DockEdge)>>,
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
            focus_body_once: AtomicBool::new(false),
            last_aot: Mutex::new(None),
            chrome_ignore: AtomicBool::new(false),
            chrome_hit: Mutex::new(None),
        }
    }

    pub fn is_applying(&self) -> bool {
        self.applying.load(Ordering::SeqCst)
    }
}

pub(crate) fn host_state(app: &AppHandle) -> Result<tauri::State<'_, FloatHost>, AppError> {
    app.try_state::<FloatHost>()
        .ok_or_else(|| AppError::InternalError {
            message: "FloatHost not managed".into(),
        })
}

pub(crate) fn lock_runtime(
    host: &FloatHost,
) -> Result<std::sync::MutexGuard<'_, RuntimeState>, AppError> {
    host.runtime.lock().map_err(|_| AppError::InternalError {
        message: "float host lock poisoned".into(),
    })
}

pub(crate) fn app_state(app: &AppHandle) -> Result<tauri::State<'_, AppState>, AppError> {
    app.try_state::<AppState>()
        .ok_or_else(|| AppError::InternalError {
            message: "AppState not managed".into(),
        })
}

pub(crate) fn companion_is_shown(app: &AppHandle) -> bool {
    chrome_window(app)
        .map(|window| window.is_visible().unwrap_or(false))
        .unwrap_or(false)
        || body_window(app)
            .map(|window| window.is_visible().unwrap_or(false))
            .unwrap_or(false)
}

pub(crate) fn companion_lost(app: &AppHandle) -> bool {
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

pub fn snapshot(app: &AppHandle) -> Result<crate::domain::CompanionSession, AppError> {
    use crate::domain::{
        derive_chrome, CompanionPlacement, CompanionSession, HomeShape, PanelMode,
    };
    use crate::repository::settings_repository::SettingsRepository;
    use crate::services::todo_service::TodoService;

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

mod appearance;
mod broadcast;
mod commands;
pub mod domain;
mod errors;
mod events;
mod float;
mod infrastructure;
mod repository;
mod scheduler;
mod services;
mod state;
mod tray_i18n;

use std::sync::Mutex;

use commands::{
    events::list_events_cmd,
    health_check,
    reminder::{
        create_reminder, delete_reminder, list_reminders, snooze_reminder, update_reminder,
    },
    settings::{get_appearance, get_settings, update_settings},
    todo::{
        create_todo, delete_todo, get_allowed_transitions, get_todo, list_all_tags,
        list_focus_board, list_todos, list_workbench_todos, restore_todo, transition_todo,
        update_todo,
    },
    window::{
        companion_click_chrome, companion_collapse_to_strip, companion_drag_ended,
        companion_minimize, companion_open_view, companion_pointer_cluster,
        companion_refresh_session, get_active_todo_count, get_companion_session,
        hide_floating_window, hide_to_tray, show_floating_window, show_main_window,
        toggle_floating_window,
    },
};
use float::FloatHost;
use infrastructure::database::Database;
use infrastructure::filesystem;
use infrastructure::notification::ensure_toast_registration;
use repository::settings_repository::SettingsRepository;
use repository::todo_repository::TodoRepository;
use state::AppState;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tray_i18n::{read_ui_locale, sync_tray_float_action, tray_labels, TrayMenuItems};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // 单实例须最先注册，避免其它插件干扰
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .setup(|app| {
            init_tracing(app.handle())?;
            ensure_toast_registration();

            let db = Database::new(app.handle())?;
            // 幂等：JSON tags → tags / todo_tags（失败则阻止启动，避免筛选与展示不一致）
            TodoRepository::backfill_normalized_tags(&db).map_err(|error| {
                tracing::error!("tag normalize backfill failed: {error}");
                error
            })?;
            app.manage(AppState { db });
            app.manage(PendingNavigation(Mutex::new(None)));
            app.manage(FloatHost::new());

            let ui_theme = SettingsRepository::get(&app.state::<AppState>().db)
                .map(|settings| settings.ui_theme)
                .unwrap_or(domain::UiTheme::System);
            let os = appearance::query_os_theme(app.handle(), None);
            let host = appearance::AppearanceHost::new(ui_theme, os);
            let resolved = host.snapshot().resolved;
            app.manage(host);
            // 尽早钉 theme，降低 WebView 第一帧 FOUC；窗口均来自 tauri.conf
            appearance::apply_theme_to_all_windows(app.handle(), resolved);

            setup_tray(app.handle())?;
            setup_main_window(app.handle())?;
            setup_companion_windows(app.handle())?;
            appearance::apply_theme_to_all_windows(app.handle(), resolved);
            apply_startup_windows(app.handle())?;
            float::update_tray_tooltip(app.handle());
            scheduler::start(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            health_check,
            create_todo,
            update_todo,
            delete_todo,
            restore_todo,
            get_todo,
            list_todos,
            list_workbench_todos,
            list_focus_board,
            list_all_tags,
            get_allowed_transitions,
            transition_todo,
            create_reminder,
            update_reminder,
            delete_reminder,
            list_reminders,
            snooze_reminder,
            get_settings,
            get_appearance,
            update_settings,
            list_events_cmd,
            hide_to_tray,
            show_main_window,
            show_floating_window,
            hide_floating_window,
            toggle_floating_window,
            get_active_todo_count,
            get_companion_session,
            companion_refresh_session,
            companion_click_chrome,
            companion_minimize,
            companion_collapse_to_strip,
            companion_drag_ended,
            companion_pointer_cluster,
            companion_open_view,
        ])
        .on_window_event(|window, event| {
            if let WindowEvent::ThemeChanged(theme) = event {
                appearance::on_os_theme_signal(window.app_handle(), *theme);
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 通知点击后待跳转的任务 ID（plugin 无 click 回调，靠窗口聚焦触发）。
pub struct PendingNavigation(pub Mutex<Option<String>>);

fn init_tracing(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let logs_dir = filesystem::logs_dir(app)?;
    std::fs::create_dir_all(&logs_dir)?;
    let log_file = logs_dir.join("app.log");
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)?;
    tracing_subscriber::fmt()
        .with_writer(Mutex::new(file))
        .with_ansi(false)
        .init();
    Ok(())
}

fn apply_startup_windows(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let Some(state) = app.try_state::<AppState>() else {
        return Ok(());
    };
    let settings = SettingsRepository::get(&state.db)?;

    if settings.autostart_enabled {
        let _ = app.autolaunch().enable();
    }

    if settings.float_auto_show {
        if let Some(main) = app.get_webview_window("main") {
            let _ = main.hide();
        }
        let _ = commands::window::show_floating_window_impl(app);
    }

    Ok(())
}

/// 关闭窗口（X）时隐藏至托盘，不退出进程；仅托盘菜单「退出」可结束进程。
fn setup_main_window(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window("main") {
        let app_handle = app.clone();
        window.on_window_event(move |event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                if let Some(win) = app_handle.get_webview_window("main") {
                    let _ = win.hide();
                }
            }
            if let WindowEvent::Focused(true) = event {
                if let Some(state) = app_handle.try_state::<PendingNavigation>() {
                    if let Ok(mut pending) = state.0.lock() {
                        if let Some(todo_id) = pending.take() {
                            let _ = app_handle.emit("navigate-to-todo", todo_id);
                        }
                    }
                }
            }
        });
    }
    Ok(())
}

fn setup_companion_windows(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    for label in [float::CHROME_LABEL, float::BODY_LABEL] {
        if let Some(window) = app.get_webview_window(label) {
            let app_handle = app.clone();
            let is_body = label == float::BODY_LABEL;
            window.on_window_event(move |event| match event {
                WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    let _ = commands::window::hide_floating_window_impl(&app_handle);
                }
                WindowEvent::Moved(_) | WindowEvent::Resized(_) => {
                    let applying = app_handle
                        .try_state::<FloatHost>()
                        .map(|host| host.is_applying())
                        .unwrap_or(false);
                    if applying {
                        return;
                    }
                    if is_body {
                        let _ = FloatHost::persist_body_if_needed(&app_handle);
                    } else {
                        let _ = FloatHost::persist_chrome_if_ball(&app_handle);
                    }
                }
                _ => {}
            });
        }
    }
    float::start_chrome_hit_test(app);
    Ok(())
}

fn setup_tray(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let locale = read_ui_locale(app);
    let labels = tray_labels(locale);

    let float_label = if float::session::companion_is_shown(app) {
        labels.hide_float
    } else {
        labels.show_float
    };
    let items = TrayMenuItems {
        float_action: MenuItem::with_id(app, "float_action", float_label, true, None::<&str>)?,
        new_todo: MenuItem::with_id(app, "new_todo", labels.new_todo, true, None::<&str>)?,
        settings: MenuItem::with_id(app, "settings", labels.settings, true, None::<&str>)?,
        quit: MenuItem::with_id(app, "quit", labels.quit, true, None::<&str>)?,
    };
    let menu = Menu::with_items(
        app,
        &[
            &items.float_action,
            &items.new_todo,
            &items.settings,
            &items.quit,
        ],
    )?;
    app.manage(items);

    let icon = app
        .default_window_icon()
        .ok_or("missing default window icon")?
        .clone();

    TrayIconBuilder::with_id("main")
        .icon(icon)
        .tooltip("Only Todo")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "float_action" => {
                if float::session::companion_is_shown(app) {
                    let _ = commands::window::hide_floating_window_impl(app);
                } else {
                    let _ = commands::window::show_floating_window_impl(app);
                }
            }
            "new_todo" => {
                let _ = commands::window::show_main_window_impl(app.clone(), None);
                let _ = app.emit("create-todo", ());
            }
            "settings" => {
                let _ = commands::window::show_main_window_impl(app.clone(), None);
                let _ = app.emit("open-settings", ());
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                let _ = commands::window::toggle_main_window_impl(app);
            }
        })
        .build(app)?;

    sync_tray_float_action(app);
    Ok(())
}

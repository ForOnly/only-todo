#![allow(dead_code, unused_imports)]

mod commands;
mod domain;
mod errors;
mod events;
mod infrastructure;
mod repository;
mod scheduler;
mod services;
mod state;

use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Mutex;

use commands::{
    health_check,
    reminder::{
        create_reminder, delete_reminder, list_reminders, snooze_reminder, update_reminder,
    },
    settings::{get_settings, update_settings},
    todo::{
        create_todo, delete_todo, get_todo, list_all_tags, list_todos, restore_todo,
        transition_todo, update_todo,
    },
    window::{
        dock_float_window, get_active_todo_count, get_float_window_state, hide_floating_window,
        hide_to_tray, is_primary_mouse_down, peek_docked_float, set_float_display_mode,
        show_floating_window, show_main_window, toggle_floating_window, try_dock_on_edge,
        undock_float_window, unpeek_docked_float, FloatGeometryLock,
    },
};
use infrastructure::database::Database;
use infrastructure::filesystem;
use infrastructure::notification::ensure_toast_registration;
use repository::settings_repository::SettingsRepository;
use state::AppState;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .setup(|app| {
            init_tracing(app.handle())?;
            ensure_toast_registration();

            let db = Database::new(app.handle())?;
            app.manage(AppState { db });
            app.manage(PendingNavigation(Mutex::new(None)));
            app.manage(FloatGeometryLock {
                locked: AtomicBool::new(false),
                generation: AtomicU64::new(0),
            });

            setup_tray(app.handle())?;
            setup_main_window(app.handle())?;
            setup_float_window(app.handle())?;
            apply_startup_windows(app.handle())?;
            commands::window::update_tray_tooltip(app.handle());
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
            list_all_tags,
            transition_todo,
            create_reminder,
            update_reminder,
            delete_reminder,
            list_reminders,
            snooze_reminder,
            get_settings,
            update_settings,
            hide_to_tray,
            show_main_window,
            show_floating_window,
            hide_floating_window,
            toggle_floating_window,
            get_active_todo_count,
            get_float_window_state,
            set_float_display_mode,
            dock_float_window,
            undock_float_window,
            try_dock_on_edge,
            peek_docked_float,
            unpeek_docked_float,
            is_primary_mouse_down,
        ])
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
        let _ = commands::window::show_floating_window_impl(app.clone());
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

fn setup_float_window(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window(commands::window::FLOAT_LABEL) {
        let app_handle = app.clone();
        window.on_window_event(move |event| {
            match event {
                WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    let _ = commands::window::hide_floating_window_impl(app_handle.clone());
                }
                WindowEvent::Moved(_) | WindowEvent::Resized(_) => {
                    if commands::window::is_geometry_locked(&app_handle) {
                        return;
                    }
                    let _ = commands::window::persist_float_bounds(&app_handle);
                }
                _ => {}
            }
        });
    }
    Ok(())
}

fn setup_tray(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let show_float =
        MenuItem::with_id(app, "show_float", "显示悬浮窗", true, None::<&str>)?;
    let show_main = MenuItem::with_id(app, "show_main", "打开主窗口", true, None::<&str>)?;
    let new_todo = MenuItem::with_id(app, "new_todo", "新建任务", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[&show_float, &show_main, &new_todo, &settings, &quit],
    )?;

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
            "show_float" => {
                let _ = commands::window::show_floating_window_impl(app.clone());
            }
            "show_main" => {
                let _ = commands::window::show_main_window_impl(app.clone(), None);
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
                let _ = commands::window::toggle_floating_window_impl(app.clone());
            }
        })
        .build(app)?;

    Ok(())
}

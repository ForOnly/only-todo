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

use std::sync::Mutex;

use commands::{
    health_check,
    reminder::{create_reminder, delete_reminder, list_reminders, update_reminder},
    settings::{get_settings, update_settings},
    todo::{
        create_todo, delete_todo, get_todo, list_todos, restore_todo, transition_todo, update_todo,
    },
    window::{hide_to_tray, show_main_window},
};
use infrastructure::database::Database;
use infrastructure::filesystem;
use infrastructure::notification::ensure_toast_registration;
use state::AppState;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            init_tracing(app.handle())?;
            ensure_toast_registration();

            let db = Database::new(app.handle())?;
            app.manage(AppState { db });
            app.manage(PendingNavigation(Mutex::new(None)));

            setup_tray(app.handle())?;
            setup_main_window(app.handle())?;
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
            transition_todo,
            create_reminder,
            update_reminder,
            delete_reminder,
            list_reminders,
            get_settings,
            update_settings,
            hide_to_tray,
            show_main_window,
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
            // 窗口聚焦时消费 PendingNavigation，通知前端定位到对应任务
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

fn setup_tray(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let show_main = MenuItem::with_id(app, "show_main", "打开主窗口", true, None::<&str>)?;
    let new_todo = MenuItem::with_id(app, "new_todo", "新建任务", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_main, &new_todo, &settings, &quit])?;

    let icon = app
        .default_window_icon()
        .ok_or("missing default window icon")?
        .clone();

    TrayIconBuilder::new()
        .icon(icon)
        .tooltip("Only Todo")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
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
                let _ = commands::window::show_main_window_impl(app.clone(), None);
            }
        })
        .build(app)?;

    Ok(())
}

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::{AppHandle, Manager};
#[cfg(target_os = "macos")]
mod menu;
mod tray;
mod clipboard;

use tauri::{utils::config::AppUrl, WindowUrl};

fn main() {
    let port = 44548;

    let mut context = tauri::generate_context!();
    let url = format!("http://localhost:{}", port).parse().unwrap();
    let window_url = WindowUrl::External(url);
    // rewrite the config so the IPC is enabled on this URL
    context.config_mut().build.dist_dir = AppUrl::Url(window_url.clone());
    context.config_mut().build.dev_path = AppUrl::Url(window_url.clone());
    let builder = tauri::Builder::default();

    #[cfg(target_os = "macos")]
    let builder = builder.menu(menu::menu());

    let builder = builder
        .system_tray(tray::system_tray())
        .on_system_tray_event(tray::system_tray_handler);

    builder
        .plugin(tauri_plugin_localhost::Builder::new(port).build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            let tray_handle = match app.tray_handle_by_id(crate::tray::TRAY_LABEL) {
                Some(h) => h,
                None => return,
            };
            let window = app.get_window("main").unwrap();

            if !window.is_visible().unwrap() || window.is_minimized().unwrap() {
                window.unminimize().unwrap();
                window.show().unwrap();
                window.set_focus().unwrap();
                tray_handle
                    .get_item("toggle")
                    .set_title("Hide Cinny")
                    .unwrap();
            }
        }))
        .invoke_handler(tauri::generate_handler![
            clipboard::clipboard_read_image,
            update_icon
        ])
        .build(context)
        .expect("error while building tauri application")
        .run(run_event_handler)
}

fn run_event_handler<R: tauri::Runtime>(app: &tauri::AppHandle<R>, event: tauri::RunEvent) {
    match event {
        tauri::RunEvent::WindowEvent { label, event, .. } => {
            tray::window_event_handler(app, &label, &event);
        }
        tauri::RunEvent::Ready => {
            let minimize = std::env::var("MINIMIZE").unwrap_or_default();
            if minimize == "1" || minimize.to_lowercase() == "true" {
                let window = app.get_window("main").unwrap();
                let tray_handle = match app.tray_handle_by_id(crate::tray::TRAY_LABEL) {
                    Some(h) => h,
                    None => return,
                };
                tray::toggle_window_state(window, tray_handle);
            }
        }
        _ => {}
    }
}

#[tauri::command]
fn update_icon(app: AppHandle, notification: bool, highlight: bool) {
    let tray_handle = app.tray_handle_by_id(tray::TRAY_LABEL).expect("Failed to get tray handle");
    let mut icon_file_name = "32x32.png";
    if notification {
        if highlight {
            icon_file_name = "cinny-highlight-32x32.png";
        }
        else {
            icon_file_name = "cinny-unread-32x32.png";
        }
    }

    let path_icon:String = format!("icons/{}", icon_file_name);
    let icon = tauri::Icon::File(app.path_resolver().resolve_resource(path_icon).unwrap());
    match tray_handle.set_icon(icon) {
        Err(e) => eprintln!("Failed to set tray icon: {:?}", e),
        Ok(_) => {},
    };
}

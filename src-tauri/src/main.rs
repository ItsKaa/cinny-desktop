#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::Manager;
#[cfg(target_os = "macos")]
mod menu;
mod tray;

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
        .build(context)
        .expect("error while building tauri application")
        .run(run_event_handler)
}

fn run_event_handler<R: tauri::Runtime>(app: &tauri::AppHandle<R>, event: tauri::RunEvent) {
    match event {
        tauri::RunEvent::WindowEvent { label, event, .. } => {
            tray::window_event_handler(app, &label, &event);
        }
        _ => {}
    }
}

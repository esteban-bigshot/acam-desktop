// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use notify_rust::Notification;
use once_cell::sync::Lazy;
use shared_child::SharedChild;
use std::env;
use std::process::Command;
use std::sync::{Arc, Mutex};
use tauri::{App, SystemTray};
use tauri::{
    CustomMenuItem, SystemTrayEvent, SystemTrayHandle, SystemTrayMenu, SystemTrayMenuItem,
    SystemTrayMenuItemHandle,
};

static JAR_NAME: &'static str = "resources/lector.jar";

struct Singleton {
    java: String,
    jar: String,
    child_process: Option<Arc<SharedChild>>,
}

impl Singleton {
    fn new() -> Self {
        Self {
            java: String::new(),
            jar: String::new(),
            child_process: None,
        }
    }

    fn set_java(&mut self, value: String) {
        self.java = value;
    }

    fn set_jar(&mut self, value: String) {
        self.jar = value;
    }

    fn set_child_process(&mut self, value: Option<Arc<SharedChild>>) {
        self.child_process = value;
    }
}

static mut GLOBALS: Lazy<Mutex<Singleton>> = Lazy::new(|| Mutex::new(Singleton::new()));

fn init_system_tray() -> SystemTray {
    let start = CustomMenuItem::new("start".to_string(), "Iniciar");
    let stop = CustomMenuItem::new("stop".to_string(), "Detener");
    let tray_menu = SystemTrayMenu::new()
        .add_item(start)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(stop);
    let tray = SystemTray::new().with_menu(tray_menu);
    return tray;
}

fn kill_prev() {
    unsafe {
        if let Some(ref mut child) = GLOBALS.lock().unwrap().child_process {
            let _ = child.kill();
        }
    }
}

fn handle_tray_event(tray: SystemTrayHandle, item_handle: SystemTrayMenuItemHandle, id: &str) {
    match id {
        "start" => {
            kill_prev();
            unsafe {
                let global = GLOBALS.lock().unwrap();
                let jar = &global.jar;
                let java = &global.java;
                let mut command = Command::new(&java);
                command.args(["-jar", &jar]);
                let shared_child_process = SharedChild::spawn(&mut command);
                match shared_child_process {
                    Ok(child) => {
                        let child_arc = Arc::new(child);
                        let child_process_listener_ref = child_arc.clone();
                        let _ =
                            std::thread::spawn(move || match child_process_listener_ref.wait() {
                                Ok(status) => {
                                    if !status.success() {
                                        if let Some(_) = status.code() {
                                            show_notification(
                                                "Error",
                                                "Se ha detenido el lector"
                                            );
                                        } else {
                                            show_notification(
                                                "Información",
                                                "Se ha forzado la detención del lector"
                                            );
                                        }
                                    }
                                }
                                Err(_) => {
                                    show_notification(
                                        "Error",
                                        "Hubo un error inesperado con el lector",
                                    );
                                }
                            });
                        GLOBALS.get_mut().unwrap().set_child_process(Some(child_arc));
                        let _ = item_handle.set_title("Reiniciar");
                        show_notification(
                            "Información",
                            "Se ha iniciado correctamente el lector"
                        );
                    }
                    Err(_) => {
                        show_notification("Error", "No fue posible iniciar el lector, es posible que no exista java en su ordenador");
                    }
                }
            }
        }
        "stop" => {
            kill_prev();
            if let Some(item) = Some(tray.get_item("start")) {
                let _ = item.set_title("Iniciar");
            }
        }
        _ => {}
    }
}

fn main() {
    unsafe {
        GLOBALS
            .lock()
            .unwrap()
            .set_java(if env::consts::OS == "windows" {
                "javaw".to_string()
            } else {
                "java".to_string()
            });
    }
    tauri::Builder::default()
        .system_tray(init_system_tray())
        .setup(|app| {
            // apply_debug(app);

            let path_buf = app
                .path_resolver()
                .resolve_resource(JAR_NAME)
                .expect("Not found");
            if path_buf.exists() {
                if let Some(path) = path_buf.to_str() {
                    unsafe {
                        GLOBALS.lock().unwrap().set_jar(path.to_string());
                    }
                }
            };
            Ok(())
        })
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => handle_tray_event(
                app.tray_handle(),
                app.tray_handle().get_item(&id),
                id.as_str(),
            ),

            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn show_notification(tittle: &str, msg: &str) {
    Notification::new()
        .summary(tittle)
        .appname("thunderbird")
        .body(msg)
        .show()
        .unwrap();
}

//[START] For dev only
// fn apply_debug(app: &mut App) {
//     #[cfg(target_os = "macos")]
//     let config = app.config();
//     let _ = notify_rust::set_application(if cfg!(feature = "custom-protocol") {
//         &(config.tauri.bundle.identifier)
//     } else {
//         "com.apple.Terminal"
//     });
// }

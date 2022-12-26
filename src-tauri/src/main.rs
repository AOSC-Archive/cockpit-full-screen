#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::Manager;

mod systemd;

fn main() {
    let sd = systemd::check_port().unwrap();
    let port = systemd::get_port(&sd).unwrap();
    let state = systemd::get_service_state_with_async().unwrap();

    if state != "running" {
        panic!("Systemd service cocket.socket not running!\nPlease check `systemctl status cockpit.socket` to get more info!")
    }

    tauri::Builder::default()
        .setup(move |app| {
            let window = app.get_window("main").unwrap();

            let js = format!("window.location.replace('http://localhost:{}')", &port);
            window.eval(&js).unwrap();

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running tauri app");
}

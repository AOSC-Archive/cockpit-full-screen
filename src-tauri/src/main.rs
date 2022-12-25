#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::process::Command;

use anyhow::{bail, Result};
use tauri::Manager;

fn main() {
    let sd = check_systemd().unwrap();
    let port = get_port(&sd).unwrap();

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

fn check_systemd() -> Result<Vec<u8>> {
    let cmd = Command::new("systemctl")
        .arg("cat")
        .arg("cockpit.socket")
        .output()?;

    if !cmd.status.success() {
        bail!("{}", std::str::from_utf8(&cmd.stderr)?)
    }

    let stdout = cmd.stdout;

    Ok(stdout)
}

fn get_port(systemd: &[u8]) -> Result<String> {
    let s = freedesktop_entry_parser::Entry::parse(systemd)?;

    let stream = s.section("Socket").attr("ListenStream");

    if stream.is_none() {
        bail!("Can not get ListenStream field!")
    }

    Ok(stream.unwrap().to_string())
}

#[test]
fn test_get_port() {
    let s = r#"# /etc/systemd/system/cockpit.socket
[Unit]
Description=Cockpit Web Service Socket
Documentation=man:cockpit-ws(8)
Wants=cockpit-motd.service

[Socket]
ListenStream=9098
ExecStartPost=-/usr/share/cockpit/motd/update-motd '' localhost
ExecStartPost=-/bin/ln -snf active.motd /run/cockpit/motd
ExecStopPost=-/bin/ln -snf inactive.motd /run/cockpit/motd

[Install]
WantedBy=sockets.target    
    "#;
    let port = get_port(s.as_bytes()).unwrap();

    assert_eq!(port, "9098")
}

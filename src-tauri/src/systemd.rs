use anyhow::{bail, Result};
use std::process::Command;
use zbus::{dbus_proxy, Connection};

#[dbus_proxy(
    interface = "org.freedesktop.systemd1.Unit",
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1/unit/cockpit_2esocket"
)]
trait SystemdUnit {
    /// SubState encodes states of the same state machine that ActiveState covers, but knows more fine-grained states that are unit-type-specific
    #[dbus_proxy(property)]
    fn sub_state(&self) -> zbus::Result<String>;
}

async fn get_state() -> zbus::Result<String> {
    let connection = Connection::system().await?;

    let proxy = SystemdUnitProxy::new(&connection).await?;
    let reply = proxy.sub_state().await?;

    Ok(reply)
}

pub fn get_service_state_with_async() -> zbus::Result<String> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .build()
        .unwrap();

    runtime.block_on(get_state())
}

pub fn check_port() -> Result<Vec<u8>> {
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

pub fn get_port(systemd: &[u8]) -> Result<String> {
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

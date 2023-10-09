use std::ffi::OsStr;
use std::process::Command;
use crate::error::{WFError, WFResult};
use crate::error::WFError::{CommandErr, HotspotCreate};

pub mod error;

pub struct Network {
    ssid: String,
    password: String,
}

impl Network {
    pub fn new(ssid: String, password: String) -> Self {
        Self {
            ssid,
            password
        }
    }
}

pub trait Client {
    fn connect(&self, network: &Network) -> WFResult<()>;
    fn disconnect(&self) -> WFResult<()>;
    fn turn_off(&self) -> WFResult<()>;
    fn turn_on(&self) -> WFResult<()>;
    fn scan(&self) -> WFResult<Vec<Network>>;
    fn is_on(&self) -> WFResult<bool>;
}

pub trait Hotspot {
    fn create(&self, network: Network) -> WFResult<()>;
    fn start(&self) -> WFResult<()>;
    fn stop(&self) -> WFResult<()>;
    fn clients(&self) -> WFResult<Vec<String>>;
    fn is_active(&self) -> WFResult<bool>;
}

pub struct WiFi {
    interface: String,
}

impl WiFi {
    pub fn new(interface: String) -> Self {
        Self {
            interface
        }
    }

    pub fn interfaces() -> Vec<String> {
        vec!["lol".to_string()]
    }

    fn command<I, S>(&self, program: &str, args: I) -> WFResult<String>
    where I: IntoIterator<Item = S>,
          S: AsRef<OsStr>
    {
        let output = Command::new(program)
            .args(args)
            .output()
            .map_err(|_| WFError::CommandIO)?;

        let err: String = String::from_utf8_lossy(&output.stderr)
            .parse()
            .map_err(|_| WFError::CommandParse)?;

        if !err.is_empty() {
            Err(CommandErr(err))?
        }

        let string: String = String::from_utf8_lossy(&output.stdout)
            .parse()
            .map_err(|_| WFError::CommandParse)?;

        Ok(string.trim().to_string())
    }
}

impl Client for WiFi {
    fn connect(&self, network: &Network) -> WFResult<()> {
        todo!()
    }

    fn disconnect(&self) -> WFResult<()> {
        todo!()
    }

    fn turn_off(&self) -> WFResult<()> {
        todo!()
    }

    fn turn_on(&self) -> WFResult<()> {
        todo!()
    }

    fn scan(&self) -> WFResult<Vec<Network>> {
        todo!()
    }

    fn is_on(&self) -> WFResult<bool> {
        todo!()
    }
}

impl Hotspot for WiFi {
    fn create(&self, network: Network) -> WFResult<()> {
        let output = self.command("nmcli", ["con", "add", "type", "wifi",
            "ifname", &self.interface, "con-name", "Hotspot", "autoconnect", "yes",
            "ssid", &network.ssid])?;

        if !output.contains("successfully added") {
            Err(HotspotCreate(output))?
        }

        let output = self.command("nmcli", ["con", "modify",
            "Hotspot", "802-11-wireless.mode", "ap", "802-11-wireless.band", "bg",
            "ipv4.method", "shared"])?;

        if !output.is_empty() {
            Err(HotspotCreate(output))?
        }

        let output = self.command("nmcli", ["con", "modify",
            "Hotspot", "wifi-sec.key-mgmt", "wpa-psk"])?;

        if !output.is_empty() {
            Err(HotspotCreate(output))?
        }

        let output = self.command("nmcli", ["con", "modify",
            "Hotspot", "wifi-sec.psk", &network.password])?;

        if !output.is_empty() {
            Err(HotspotCreate(output))?
        }

        Ok(())
    }

    fn start(&self) -> WFResult<()> {
        let output = self.command("nmcli", ["con", "up", "Hotspot"])?;

        if !output.contains("Connection successfully activated") {
            Err(HotspotCreate(output))?
        }

        Ok(())
    }

    fn stop(&self) -> WFResult<()> {
        let output = self.command("nmcli", ["con", "down", "Hotspot"])?;

        if !output.contains("successfully deactivated") {
            Err(HotspotCreate(output))?
        }

        Ok(())
    }

    fn clients(&self) -> WFResult<Vec<String>> {
        todo!()
    }

    fn is_active(&self) -> WFResult<bool> {
        todo!()
    }
}

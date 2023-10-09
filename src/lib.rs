use std::ffi::OsStr;
use std::process::Command;
use crate::error::{WFError, WFResult};
use crate::error::WFError::{CommandErr, HotspotCreate, WifiAction};

pub mod error;

#[derive(Debug)]
pub struct Network {
    bssid: String,
    ssid: String,
    mode: String,
    channel: u32,
    rate: String,
    signal: u32,
    security: String,
}


pub trait Client {
    fn connect(&self, ssid: String, password: String) -> WFResult<()>;
    fn disconnect(&self) -> WFResult<()>;
    fn turn_off(&self) -> WFResult<()>;
    fn turn_on(&self) -> WFResult<()>;
    fn scan(&self) -> WFResult<Vec<Network>>;
    fn is_on(&self) -> WFResult<bool>;
}

pub trait Hotspot {
    fn create(&self, ssid: String, password: Option<String>) -> WFResult<()>;
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
    fn connect(&self, ssid: String, password: String) -> WFResult<()> {
        let output = self.command("nmcli", ["device", "wifi", "connect",
            &ssid, "password", &password, "ifname", &self.interface])?;

        if !output.contains("successfully activated") {
            Err(WifiAction(output))?
        }

        Ok(())
    }

    fn disconnect(&self) -> WFResult<()> {
        let output = self.command("nmcli", ["device", "disconnect", "wlan0"])?;

        if !output.contains("successfully disconnected") {
            Err(WifiAction(output))?
        }

        Ok(())
    }

    fn turn_off(&self) -> WFResult<()> {
        todo!()
    }

    fn turn_on(&self) -> WFResult<()> {
        todo!()
    }

    fn scan(&self) -> WFResult<Vec<Network>> {
        let output = self.command("nmcli", ["device", "wifi", "rescan",
            "ifname", &self.interface])?;

        if !output.is_empty() {
            Err(WifiAction(output))?
        }

        let output = self.command("nmcli", ["device", "wifi", "list",
            "ifname", &self.interface])?;

        let output = output.lines().collect::<Vec<&str>>();
        let mut output = output.iter();

        let mut words: Vec<String> = Vec::new();
        let mut current_word = String::new();
        let mut in_whitespace = false;
        let header = output.next().unwrap();

        for c in header.chars() {
            if c.is_whitespace() {
                current_word.push(c);
                in_whitespace = true;
            } else {
                if in_whitespace {
                    words.push(current_word.clone());
                    current_word.clear();
                }

                current_word.push(c);
                in_whitespace = false;
            }
        }

        // Add the last word if there's one
        if !current_word.is_empty() {
            words.push(current_word.clone());
        }

        let mut networks = Vec::new();
        for line in output.by_ref() {
            let mut index = words[0].len();
            let bssid = line[index..index + words[1].len()].trim().to_string();
            index += words[1].len();
            let ssid = line[index..index + words[2].len()].trim().to_string();
            index += words[2].len();
            let mode = line[index..index + words[3].len()].trim().to_string();
            index += words[3].len();
            let channel = line[index..index + words[4].len()].trim().to_string().parse().unwrap_or(0);
            index += words[4].len();
            let rate = line[index..index + words[5].len()].trim().to_string();
            index += words[5].len();
            let signal = line[index..index + words[6].len()].trim().to_string().parse().unwrap_or(0);
            index += words[6].len() + words[7].len();
            let security = line[index..].trim().to_string();

            networks.push(Network {
                bssid,
                ssid,
                mode,
                channel,
                rate,
                signal,
                security,
            })
        }

        Ok(networks)
    }

    fn is_on(&self) -> WFResult<bool> {
        todo!()
    }
}

impl Hotspot for WiFi {
    fn create(&self, ssid: String, password: Option<String>) -> WFResult<()> {
        let output = self.command("nmcli", ["con", "add", "type", "wifi",
            "ifname", &self.interface, "con-name", "Hotspot", "autoconnect", "yes",
            "ssid", &ssid])?;

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
            "Hotspot", "wifi-sec.psk", &password.unwrap_or("parola_default".to_string())])?;

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

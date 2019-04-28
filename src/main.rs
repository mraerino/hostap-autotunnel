use crossbeam_channel::{bounded, select, tick, Receiver};
use eui48::MacAddress;
use std::time::Duration;

mod wpactrl;
use wpactrl::{WpaCtrl, WpaCtrlAttached};

type IntError = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, IntError>;

const SOCK_PATH: &str = "/var/run/hostapd/wlan0";

fn main() {
    let mut log = HostAPMonitor::connect(SOCK_PATH).unwrap();
    println!("Attached");

    println!("pinging...");
    log.ping().unwrap();
    println!("ping success");

    log.log_events().unwrap();
}

fn ctrl_channel() -> Result<Receiver<()>> {
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}

pub type HostAPMonitor = WpaCtrlAttached;

impl HostAPMonitor {
    fn connect(path: &str) -> Result<Self> {
        let wpa = WpaCtrl::new().ctrl_path(path).open()?;
        let conn = wpa.attach()?;
        Ok(conn)
    }

    fn ping(&mut self) -> Result<()> {
        let resp = self.request("PING")?;
        match resp.as_str() {
            "PONG\n" => Ok(()),
            _ => Err("invalid reply".into()),
        }
    }

    fn log_events(&mut self) -> Result<()> {
        let ctrl_c_events = ctrl_channel()?;
        let ticks = tick(Duration::from_secs(1));

        loop {
            select! {
                recv(ticks) -> _ => {
                    match self.recv().unwrap() {
                        Some(s) => {
                            let evt = Event::from_string(&s)?;
                            match evt {
                                Event::StationConnected{ mac } => {
                                    println!("Got new station: {}", mac);
                                    let info = self.station_info(mac)?;
                                    println!("Info: {}", info);
                                }
                                _ => {}
                            }
                        }
                        None => std::thread::sleep(std::time::Duration::from_millis(10)),
                    }
                }
                recv(ctrl_c_events) -> _ => {
                    println!("Exiting. Bye 👋");
                    break
                }
            }
        }

        Ok(())
    }

    fn station_info(&mut self, mac: MacAddress) -> Result<String> {
        let req = format!("STA {}", mac.to_string(eui48::MacAddressFormat::HexString));
        println!("CMD: {}", req);
        let resp = self.request(req.as_str())?;
        Ok(resp)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Event {
    StationConnected { mac: MacAddress },
    StationDisconnected { mac: MacAddress },
    Other { ident: String, args: Vec<String> },
}

impl Event {
    fn from_string(event: &str) -> Result<Self> {
        let mut event_parts = event.split(" ");

        let event_ident: String = event_parts
            .next()
            .unwrap_or("")
            .chars()
            .skip(if event.starts_with("<") {
                event.find(">").map(|p| p + 1).unwrap_or(0)
            } else {
                0
            })
            .take_while(|c| *c != ' ')
            .collect();

        if event_ident == "" {
            return Err("empty identifier".into());
        }

        Ok(match (event_ident.as_str(), event_parts.next()) {
            ("AP-STA-CONNECTED", Some(mac)) => Event::StationConnected {
                mac: MacAddress::parse_str(mac)?,
            },
            ("AP-STA-DISCONNECTED", Some(mac)) => Event::StationDisconnected {
                mac: MacAddress::parse_str(mac)?,
            },
            (ident, _) => Event::Other {
                ident: ident.to_owned(),
                args: event_parts.map(|s| s.to_owned()).collect(),
            },
        })
    }
}

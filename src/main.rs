use crossbeam_channel::{bounded, select, tick, Receiver};
use eui48::MacAddress;
use std::time::Duration;

mod wpactrl;
use wpactrl::WpaCtrl;

type IntError = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, IntError>;

const SOCK_PATH: &str = "/var/run/hostapd/wlan0";

fn main() {
    log_events().unwrap();
}

fn ctrl_channel() -> Result<Receiver<()>> {
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}

fn log_events() -> Result<()> {
    println!("Using path: {:?}", SOCK_PATH);
    let wpa = WpaCtrl::new().ctrl_path(SOCK_PATH).open()?;
    let mut receiver = wpa.attach()?;

    println!("Attached");

    let ctrl_c_events = ctrl_channel()?;
    let ticks = tick(Duration::from_secs(1));

    loop {
        select! {
            recv(ticks) -> _ => {
                match receiver.recv().unwrap() {
                    Some(s) => {
                        let evt = Event::from_string(&s);
                        println!("EVENT: {:?}", evt)
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
    receiver.detach().unwrap();

    Ok(())
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
                event.find(">").map(|p| p+1).unwrap_or(0)
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

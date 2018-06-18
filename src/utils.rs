// this file contains some helpers that we want to drop eventually

use std::process::Command;
use wifi::Network;
use errors::Result;

fn cmd(cmd: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(cmd)
        .args(args)
        .output()?;
    if !output.status.success() {
        bail!("command exited with error");
    }

    let output = String::from_utf8(output.stdout)?;
    Ok(output)
}

fn parse_essid_from_iwconfig(output: &str) -> Option<String> {
    if let Some(idx) = output.find("ESSID:") {
        let output = &output[idx + 7..];

        if let Some(idx) = output.find('"') {
            return Some(output[..idx].to_string());
        }
    }

    None
}

pub fn current_essid(iface: &str) -> Result<String> {
    let output = cmd("iwconfig", &[iface])?;
    match parse_essid_from_iwconfig(&output) {
        Some(ssid) => Ok(ssid),
        None => bail!("ssid for interface not found"),
    }
}

pub fn scan_wifi(iface: &str) -> Result<Vec<Network>> {
    let output = cmd("iwlist", &[iface, "scan"])?;
    parse_scan_output(&output)
}

pub fn parse_scan_output(output: &str) -> Result<Vec<Network>> {
    use regex::Regex;

    let re = Regex::new(r"^\s+Cell \d+ - Address: ([0-9A-F:]+)$").unwrap();
    let signal_re = Regex::new(r"^\s*Quality=([\d/]+)\s+Signal level=(\-\d+) dBm").unwrap();

    let mut networks = Vec::new();
    let mut ap = None;
    let mut essid = None;
    let mut encryption = None;
    let mut quality = None;
    let mut signal = None;
    let mut channel = None;
    let mut mode = None;

    for line in output.split("\n") {
        if !line.starts_with(" ") {
            continue;
        }
        if let Some(cell) = re.captures(line) {
            if ap.is_some() {
                networks.push(Network::build(&mut ap,
                                             &mut essid,
                                             &mut encryption,
                                             &mut quality,
                                             &mut signal,
                                             &mut channel,
                                             &mut mode));
            }

            ap = Some(cell.get(1).unwrap().as_str().to_string());
            // println!("got ap {:?}", ap);
        } else {
            let trimmed = line.trim_left();

            if trimmed.starts_with("Encryption key:") {
                encryption = Some(String::from(&trimmed[15..]));
                // println!("\tencryption: {:?}", encryption);
            } else if trimmed.starts_with("ESSID:") {
                essid = Some(String::from(&trimmed[7..trimmed.len()-1]));
                // println!("\tessid={:?}", essid);
            } else if trimmed.starts_with("Channel:") {
                channel = Some(trimmed[8..].parse::<u16>().unwrap());
                // println!("\tchannel={:?}", channel);
            } else if trimmed.starts_with("Mode:") {
                mode = Some(String::from(&trimmed[5..]));
                // println!("\tmode={:?}", mode);
            } else if trimmed.starts_with("Quality=") {
                let cap = signal_re.captures(line).expect("regex didn't match");

                quality = Some(String::from(cap.get(1).unwrap().as_str()));
                signal = Some({
                    let signal = cap.get(2).unwrap().as_str();
                    signal.parse().unwrap()
                });

                // println!("\tquality={:?} signal={:?}", quality, signal);
            } else if trimmed.starts_with("IE: Unknown: ") {
                // ignore unknown extension
            } else {
                // println!("{:?}", line);
            }
        }
    }

    if ap.is_some() {
        networks.push(Network::build(&mut ap,
                                     &mut essid,
                                     &mut encryption,
                                     &mut quality,
                                     &mut signal,
                                     &mut channel,
                                     &mut mode));
    }

    Ok(networks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_essid() {
        let data = "wlp3s0    IEEE 802.11  ESSID:\"this is my ssid\"\nsome garbage";

        let essid = parse_essid_from_iwconfig(data);
        assert_eq!(essid, Some(String::from("this is my ssid")));
    }

    #[test]
    fn test_parse_scan() {
        let output = include_str!("../tests/iwlist.txt");
        let result = parse_scan_output(output).unwrap();
        println!("{:#?}", result);
    }
}

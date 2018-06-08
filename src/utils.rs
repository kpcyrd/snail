// this file contains some helpers that we want to drop eventually

use std::process::Command;
use ::Result;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_essid() {
        let data = "wlp3s0    IEEE 802.11  ESSID:\"this is my ssid\"\nsome garbage";

        let essid = parse_essid_from_iwconfig(data);
        assert_eq!(essid, Some(String::from("this is my ssid")));
    }
}

// android captive portal detection url
// https://stackoverflow.com/questions/13958614/how-to-check-for-unrestricted-internet-access-captive-portal-detection
// TODO: this should be configurable
const PROBE_WALLED_GARDEN_URL: &str = "http://clients3.google.com/generate_204";

use dns::Resolver;
use errors::Result;
use scripts::Loader;
use web::Client;
use wifi::NetworkStatus;

use std::net::IpAddr;


#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct WalledGardenFingerprint {
    // The redirect we got for our probe
    pub redirect: Option<String>,
    // The portal we arrived at after following redirects
    // pub portal: Option<String>,
}

pub fn detect_walled_garden(resolver: Resolver) -> Result<Option<WalledGardenFingerprint>> {
    let client = Client::new(resolver);
    let req = client.get(PROBE_WALLED_GARDEN_URL)?;

    if req.status == 204 {
        info!("got 204 reply");
        Ok(None)
    } else {
        let redirect = match req.headers.get("location") {
            Some(redirect) => {
                Some(redirect.to_string())
            },
            None => {
                warn!("no redirect detected?!");
                None
            },
        };

        Ok(Some(WalledGardenFingerprint {
            redirect,
            // portal: None,
        }))
    }
}

pub fn decap(status: &mut NetworkStatus, recursors: &[IpAddr]) -> Result<()> {
    // TODO: dns server could be empty
    let resolver = Resolver::with_udp(recursors)?;
    match detect_walled_garden(resolver) {
        Ok(Some(fingerprint)) => {
            status.set_uplink_status(Some(false));
            info!("detected captive portal: {:?}", fingerprint);

            let loader = Loader::from_status(&Some(status.clone()))?;
            if let Some(ssid) = status.ssid.clone() {
                let scripts = loader.load_all_scripts()?;
                info!("loaded {} scripts", scripts.len());

                let mut solved = false;
                for script in scripts {
                    if script.detect_network(&ssid)? {
                        info!("trying {:?}", script.descr());

                        match script.decap() {
                            Ok(_) => {
                                info!("script reported success, probing network");
                                status.set_uplink_status(Some(true));
                                let resolver = Resolver::with_udp(recursors)?;
                                match detect_walled_garden(resolver) {
                                    Ok(Some(_)) => {
                                        warn!("captive portal is still active");
                                    },
                                    Ok(None) => {
                                        status.set_uplink_status(Some(true));
                                        status.script_used = Some(script.descr().to_string());
                                        info!("working internet detected");
                                        solved = true;
                                        break;
                                    },
                                    Err(err) => {
                                        warn!("captive portal test failed: {}", err);
                                    },
                                }
                            },
                            Err(err) => {
                                warn!("script reported error: {}", err);
                            },
                        };
                    }
                }

                if !solved {
                    status.set_uplink_status(Some(false));
                    info!("no scripts left, giving up");
                }
            } else {
                info!("decap engine is only enabled on wireless networks");
            }
        },
        Ok(None) => {
            status.set_uplink_status(Some(true));
            info!("working internet detected");
        },
        Err(err) => {
            warn!("captive portal test failed: {}", err);
            status.set_uplink_status(Some(false));
        },
    }

    Ok(())
}

// android captive portal detection url
// https://stackoverflow.com/questions/13958614/how-to-check-for-unrestricted-internet-access-captive-portal-detection
// TODO: this should be configurable
const PROBE_WALLED_GARDEN_URL: &str = "http://clients3.google.com/generate_204";

use web::Client;
use dns::Resolver;
use ::Result;


#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct WalledGardenFingerprint {
    // The redirect we got for our probe
    pub redirect: Option<String>,
    // The portal we arrived at after following redirects
    // pub portal: Option<String>,
}

pub fn detect_walled_garden(resolver: Resolver) -> Result<Option<WalledGardenFingerprint>> {
    // TODO: use dns from snailctl status
    let client = Client::new(resolver);
    let req = client.get(PROBE_WALLED_GARDEN_URL)?;
    // let req = client.get("https://httpbin.org/redirect-to?url=/anything&status=302")?;

    if req.status == 204 {
        info!("got 204 reply");
        Ok(None)
    } else {
        let redirect = match req.headers.get("location") {
            Some(redirect) => {
                println!("got redirect! {:?}", redirect);
                Some(redirect.to_string())
            },
            None => {
                println!("no redirect detected?!");
                None
            },
        };

        // TODO: return captive portal report
        Ok(Some(WalledGardenFingerprint {
            redirect,
            // portal: None,
        }))
    }
}

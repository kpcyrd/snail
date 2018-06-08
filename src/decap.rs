// android captive portal detection url
// https://stackoverflow.com/questions/13958614/how-to-check-for-unrestricted-internet-access-captive-portal-detection
// TODO: this should be configurable
const PROBE_WALLED_GARDEN_URL: &str = "http://clients3.google.com/generate_204";

use ::Result;
use reqwest::{self, StatusCode};


pub fn detect_walled_garden() -> Result<Option<()>> {
    info!("sending request to {:?}", PROBE_WALLED_GARDEN_URL);
    let req = reqwest::get(PROBE_WALLED_GARDEN_URL)?;

    if req.status() == StatusCode::NoContent {
        info!("got 204 reply");
        Ok(None)
    } else {
        // TODO: return captive portal report
        Ok(Some(()))
    }
}

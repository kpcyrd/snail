#[derive(Debug)]
pub struct Network {
    pub ap: String,
    pub essid: String,
    pub encryption: String,
    pub quality: String,
    pub signal: i32,
    pub channel: u16,
    pub mode: String,
}

impl Network {
    pub fn build(ap: &mut Option<String>,
                 essid: &mut Option<String>,
                 encryption: &mut Option<String>,
                 quality: &mut Option<String>,
                 signal: &mut Option<i32>,
                 channel: &mut Option<u16>,
                 mode: &mut Option<String>) -> Self {
        Network {
            ap: ap.take().unwrap_or(String::new()),
            essid: essid.take().unwrap_or(String::new()),
            encryption: encryption.take().unwrap_or(String::new()),
            quality: quality.take().unwrap_or(String::new()),
            signal: signal.take().unwrap_or(0),
            channel: channel.take().unwrap_or(0),
            mode: mode.take().unwrap_or(String::new()),
        }
    }
}

use structopt::clap::AppSettings;


#[derive(StructOpt, Debug)]
#[structopt(author = "",
            raw(global_settings = "&[AppSettings::ColoredHelp]"))]
pub struct Args {
    #[structopt(short = "v", long = "verbose",
                raw(global = "true"), parse(from_occurrences),
                help="Verbose output")]
    pub verbose: u8,

    #[structopt(short="S", long="socket", default_value="ipc:///tmp/snail.sock",
                help="snaild socket path")]
    pub socket: String,

    #[structopt(subcommand)]
    pub subcommand: Option<SubCommand>,
}

#[derive(StructOpt, Debug)]
pub enum SubCommand {
    #[structopt(author = "",
                name="scan",
                about="Show nearby networks, needs root to trigger a scan")]
    Scan(Scan),
    #[structopt(author = "",
                name="decap",
                about="Manually run captive portal bypass")]
    Decap(Decap),
    #[structopt(author = "",
                name="status",
                about="Show current network status")]
    Status(Status),
    #[structopt(author = "",
                name="dns",
                about="Run dns request inside target network")]
    Dns(Dns),
    #[structopt(author = "",
                name="http",
                about="Run http request inside target network")]
    Http(Http),
}

#[derive(StructOpt, Debug)]
pub struct Scan {
    pub interface: String,
}

#[derive(StructOpt, Debug)]
pub struct Decap {
    #[structopt(long="dns",
                help="Overwrite default dns servers")]
    pub dns: Option<String>,
    #[structopt(short="s", long="snaild",
                help="Use snaild status")]
    pub snaild: bool,
    #[structopt(short="f", long="skip-check",
                help="Don't check for captive portal")]
    pub skip_check: bool,
    #[structopt(help="Use specific script instead of auto detection")]
    pub script: Option<String>,
}

#[derive(StructOpt, Debug)]
pub struct Status {

}

#[derive(StructOpt, Debug)]
pub struct Dns {
    pub query: String,
    pub record: Option<String>,
}

#[derive(StructOpt, Debug)]
pub struct Http {

}

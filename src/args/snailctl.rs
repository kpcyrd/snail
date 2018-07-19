use structopt::clap::AppSettings;
use std::net::IpAddr;


#[derive(StructOpt, Debug)]
#[structopt(author = "",
            raw(global_settings = "&[AppSettings::ColoredHelp]"))]
pub struct Args {
    #[structopt(short = "v", long = "verbose",
                raw(global = "true"), parse(from_occurrences),
                help="Verbose output")]
    pub verbose: u8,

    #[structopt(short="S", long="socket",
                help="snaild socket path")]
    pub socket: Option<String>,

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
    #[structopt(author = "",
                name="connect",
                about="Open tcp connection inside target network")]
    Connect(Connect),
    #[structopt(author = "",
                name="doh",
                about="Resolve a dns name with dns-over-https")]
    Doh(Dns),
    #[structopt(author = "",
                name="bash-completion",
                about="Generate bash completion script for the snailctl command")]
    BashCompletion,
}

#[derive(StructOpt, Debug)]
pub struct Scan {
    pub interface: String,
}

#[derive(StructOpt, Debug)]
pub struct Decap {
    #[structopt(long="dns",
                help="Overwrite default dns servers")]
    pub dns: Vec<IpAddr>,
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
    #[structopt(long="json",
                help="Report as json")]
    pub json: bool,
}

#[derive(StructOpt, Debug)]
pub struct Dns {
    #[structopt(help="The record you want to query")]
    pub query: String,
    #[structopt(default_value="A",
                help="The query type you want to lookup")]
    pub record: String,
}

#[derive(StructOpt, Debug)]
pub struct Http {
    #[structopt(help="Request url")]
    pub url: String,
    #[structopt(short="X", long="method", default_value="GET",
                help="Set http request method")]
    pub method: String,
}

#[derive(StructOpt, Debug)]
pub struct Connect {
    #[structopt(help="Destination host")]
    pub host: String,
    #[structopt(help="Destination port")]
    pub port: u16,
}

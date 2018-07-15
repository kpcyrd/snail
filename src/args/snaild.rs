use structopt::clap::AppSettings;

#[derive(StructOpt, Debug)]
#[structopt(author = "",
            raw(global_settings = "&[AppSettings::ColoredHelp]"))]
pub struct Args {
    #[structopt(short = "v", long = "verbose",
                raw(global = "true"),
                help="Verbose output")]
    pub verbose: bool,

    #[structopt(short="S", long="socket",
                help="snaild socket path")]
    pub socket: Option<String>,

    #[structopt(subcommand)]
    pub subcommand: Option<SubCommand>,
}

#[derive(StructOpt, Debug)]
pub enum SubCommand {
    #[structopt(author = "",
                name="start",
                about="Start daemon")]
    Start(Start),
    #[structopt(author = "",
                name="dhcp",
                about="Start dhcp daemon")]
    Dhcp(Dhcp),
    #[structopt(author = "",
                name="decap",
                about="Start decap daemon")]
    Decap,
    #[structopt(author = "",
                name="dns",
                about="Start dns resolver")]
    Dns(Dns),
}

#[derive(StructOpt, Debug)]
pub struct Start {
    pub interface: String,
}

#[derive(StructOpt, Debug)]
pub struct Dhcp {
    pub interface: String,
}

#[derive(StructOpt, Debug)]
pub struct Dns {
    // pub interface: String,
}

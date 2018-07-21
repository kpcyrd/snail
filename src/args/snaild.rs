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
    #[structopt(author = "",
                name="vpnd",
                about="Start vpn server daemon")]
    Vpnd(Vpnd),
    #[structopt(author = "",
                name="vpn",
                about="Start vpn client daemon")]
    Vpn(Vpn),
    #[structopt(author = "",
                name="vpn-keygen",
                about="Generate a keypair for vpn")]
    VpnKeyGen(VpnKeyGen),
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

#[derive(StructOpt, Debug)]
pub struct Vpnd {
    pub interface: String,
}

#[derive(StructOpt, Debug)]
pub struct Vpn {
}

#[derive(StructOpt, Debug)]
pub struct VpnKeyGen {
}

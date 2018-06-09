use structopt::clap::AppSettings;

#[derive(StructOpt, Debug)]
#[structopt(author = "",
            raw(global_settings = "&[AppSettings::ColoredHelp]"))]
pub struct Args {
    #[structopt(short = "v", long = "verbose",
                raw(global = "true"),
                help="Verbose output")]
    pub verbose: bool,

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
                about="Automatically bypass captive portal")]
    Decap(Decap),
}

#[derive(StructOpt, Debug)]
pub struct Scan {
    pub interface: String,
}

#[derive(StructOpt, Debug)]
pub struct Decap {

}

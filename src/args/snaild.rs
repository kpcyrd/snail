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

    pub interface: String,
}

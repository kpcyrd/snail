use structopt::clap::AppSettings;

#[derive(StructOpt, Debug)]
#[structopt(author = "",
            raw(global_settings = "&[AppSettings::ColoredHelp]"))]
pub struct Args {
    #[structopt(short = "v", long = "verbose",
                raw(global = "true"),
                help="Verbose output")]
    pub verbose: bool,

    #[structopt(short="S", long="socket", default_value="ipc:///tmp/snail.sock",
                help="snaild socket path")]
    pub socket: String,

    pub interface: String,
}



use clap::App;
use clap::Arg;

//mod error;

//pub use self::error::Error;
//pub use self::error::ErrorKind;
//pub use self::error::Result;

/// Process command line arcuments and run the given command.
pub fn run() -> Result<(), std::io::Error> {
    // Initialise clap.
    let version = format!(
        "{} [{}; {}]",
        env!("CARGO_PKG_VERSION"),
        env!("GIT_BUILD_HASH"),
        env!("GIT_BUILD_TAINT"),
    );
    let _args = App::new("replictl")
        .version(version.as_ref())
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .default_value("replicante.yaml")
                .takes_value(true)
                .global(true)
                .help("Specifies the configuration file to use"),
        )
        .get_matches();

    // TODO
    panic!("TODO: implement new replictl")
}

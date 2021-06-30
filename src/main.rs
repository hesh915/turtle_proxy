use slog::{crit, debug, error, info, trace, warn, Drain};

mod local;
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let logger = slog::Logger::root(drain, slog::o!());

    crit!(logger, "test crit");
    error!(logger, "test error");
    warn!(logger, "test warn");
    info!(logger, "test info");
    debug!(logger, "test debug");
    trace!(logger, "test trace");

    let matches = clap::App::new("turtle proxy")
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about("a turtle proxy")
        .usage(
            "turtle_proxy [FLAGS] [OPTIONS]\n    turtle_proxy  -c -l <LISTEN> -r <REMOTE>\n    turtle_proxy  -s -l <LISTEN>",
        )
        .arg(
            clap::Arg::with_name("CLIENT")
                .short("c")
                .long("client")
                .required_unless("SERVER")
                .conflicts_with("SERVER")
                .help("local client mode proxy"),
        )
        .arg(
            clap::Arg::with_name("SERVER")
                .short("s")
                .long("server")
                .required_unless("CLIENT")
                .conflicts_with("CLIENT")
                .help("remote server mode proxy"),
        )
        .arg(
            clap::Arg::with_name("LISTEN")
                .short("l")
                .long("listen")
                .help("client or server mode listen local port")
                .default_value_if("CLIENT", None, "0.0.0.0:10801")
                .default_value_if("SERVER", None, "0.0.0.0:10802")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("REMOTE")
                .short("r")
                .long("remote")
                .help("client mode connect remote tunnel addr")
                .default_value_if("CLIENT", None, "192.168.50.155:22")
                .takes_value(true),
        )
        .get_matches();

    if matches.is_present("CLIENT") {
        let listene_addr = matches.value_of("LISTEN").unwrap();
        let server_addr = matches.value_of("REMOTE").unwrap();

        local::start_local(&logger, listene_addr, server_addr).await?;
    } else if matches.is_present("SERVER") {
        //todo server
    }

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        //info!(logger, "heart beat !");
    }
}

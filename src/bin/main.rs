use std::fs;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate error_chain;
use error_chain::ChainedError;

#[macro_use]
extern crate log;

extern crate serde;
extern crate serde_json;

extern crate tincmgr;
use tincmgr::logging::init_logger;
use tincmgr::tinc_tcp_stream::TincStream;
use std::path::{PathBuf, Path};
use tincmgr::domain::Data;
use std::thread::sleep;
use std::time::Duration;

const DEFAULT_DATA_SAVE_PATH: &str = "/var/www/html/data/nodes.json";
const DEFAULT_PIDFILE_PATH: &str = "/root/tinc/tinc.pid";
const DEFAULT_LOG_DIR: &str = "/var/log/tincmgr";
const DEFAULT_LOG_FILE: &str = "/tincmgr.log";

error_chain! {
    errors {
        LogError(msg: &'static str) {
            description("Error setting up log")
            display("Error setting up log: {}", msg)
        }
        CreateStreamError(msg: &'static str) {
            description("Error create TCP stream with tincd")
            display("Error: {}", msg)
        }
        WriteJsonError(msg: &'static str) {
            description("Error when write json to file")
            display("Error: {}", msg)
        }
    }
    links {
//        tincmgr::logging::WriteFileError(path: PathBuf);
    }
}


fn main() {
    let exit_code = match run() {
        Ok(_) => 0,
        Err(error) => {
            if let ErrorKind::LogError(_) = error.kind() {
                eprintln!("{}", error.display_chain());
            } else {
                error!("{}", error.display_chain());
            }
            1
        }
    };
    debug!("Process exiting with code {}", exit_code);
    ::std::process::exit(exit_code);
}

fn run() -> Result<()> {
    let app = clap::App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .arg(
            clap::Arg::with_name("debug")
                .short("d")
                .long("debug")
                .takes_value(true)
                .value_name("level")
                .help("Increase debug level or set it to LEVEL."),
        )
        .arg(
            clap::Arg::with_name("outfile")
                .short("o")
                .long("outfile")
                .takes_value(true)
                .value_name("path")
                .help("Read configuration options from DIR."),
        )
        .arg(
            clap::Arg::with_name("pidfile")
                .short("p")
                .long("pidfile")
                .takes_value(true)
                .value_name("path")
                .help("Write PID and control socket cookie to FILENAME."),
        )
        .get_matches();

    let log_level = match app.value_of("debug") {
        Some(log_level) => {
            match log_level {
                _ if log_level == "0" => log::LevelFilter::Error,
                _ if log_level == "1" => log::LevelFilter::Warn,
                _ if log_level == "2" => log::LevelFilter::Info,
                _ if log_level == "3" => log::LevelFilter::Debug,
                _ if log_level == "4" => log::LevelFilter::Trace,
                _  => log::LevelFilter::Off,
            }
        }
        None => log::LevelFilter::Off,
    };

    let outfile = match app.value_of("outfile") {
        Some(dir) => dir,
        None => DEFAULT_DATA_SAVE_PATH,
    };

    let pidfile = match app.value_of("pidfile") {
        Some(pidfile) => pidfile,
        None => DEFAULT_PIDFILE_PATH,
    };

    init_logger(
        log_level,
        Some(&PathBuf::from(DEFAULT_LOG_DIR)),
        Some(&PathBuf::from(DEFAULT_LOG_FILE)),
        true,
    )
    .chain_err(|| ErrorKind::LogError("Unable to initialize logger"))?;

    main_loop(pidfile, outfile)?;
    Ok(())

}

fn main_loop(
    pidfile:    &str,
    outfile:    &str,
) -> Result<()> {
    loop {
        debug!("Start fresh.");
        let data = get_data(pidfile)?;
        write_json(outfile, data)?;
        debug!("Finnish fresh.");
        sleep(Duration::from_millis(20000));
    }
}

fn get_data(pid_path: &str) -> Result<String> {
    let mut _data_str = String::new();

    loop {
        let mut tinc_stream = TincStream::new(pid_path).expect("123");

        if let Ok(nodes) = tinc_stream.dump_nodes() {
            debug!("dump_nodes.");
            if let Ok(edges) = tinc_stream.dump_edges() {
                debug!("dump_edges.");
                if let Ok(subnets) = tinc_stream.dump_subnets() {
                    debug!("dump_subnets.");
                    let data = Data::new(
                        nodes,
                        subnets,
                        edges);
                    if let Ok(data) = serde_json::to_string(&data) {
                        _data_str = data;
                        if let Ok(_) = tinc_stream.purge() {
                            ()
                        }
                        break;
                    }
                }
            }
        }
        sleep(Duration::from_millis(1000));
    }
    Ok(_data_str)
}

fn write_json(file_path: &str, data: String) -> Result<()> {
    let path = Path::new(file_path);
    if !path.is_file() {
        let _ = fs::File::create(path);
    }
    fs::write(file_path, data)
        .chain_err(|| ErrorKind::WriteJsonError("Error when write json to file"))?;
    Ok(())
}
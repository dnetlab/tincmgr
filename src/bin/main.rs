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
use tincmgr::domain::Data;
use tincmgr::web_server::web_server;

use std::env;
use std::thread::sleep;
use std::thread::spawn;
use std::time::Duration;
use std::path::{PathBuf, Path};
use std::fs;

const DEFAULT_PIDFILE_PATH: &str = "/root/tinc/tinc.pid";
const DEFAULT_LOG_DIR: &str = "/var/log/tincmgr";
const DEFAULT_LOG_FILE: &str = "/tincmgr.log";
const DEFAULT_WEB_SERVER_PORT: &str = "8080";

error_chain! {
    errors {
        LogError(msg: &'static str) {
            description("Error setting up log")
            display("Error setting up log: {}", msg)
        }
        CreateStreamError(msg: &'static str) {
            description("Error create TCP stream with tincd")
            display("{}", msg)
        }
        WriteJsonError(msg: &'static str) {
            description("Error when write json to file")
            display("{}", msg)
        }
        NotFindTincPiDError(msg: &'static str) {
            description("Error find tinc pid file")
            display("{}", msg)
        }
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
            clap::Arg::with_name("pidfile")
                .short("f")
                .long("pidfile")
                .takes_value(true)
                .value_name("path")
                .help(
                    &format!(
                        "PID and control socket cookie FILENAME.\ndefualt:{}",
                        DEFAULT_PIDFILE_PATH
                    ),
                )
        )
        .arg(
            clap::Arg::with_name("port")
                .short("p")
                .long("port")
                .takes_value(true)
                .value_name("port")
                .help(
                    &format!(
                        "Web server port.\ndefualt:{}",
                        DEFAULT_WEB_SERVER_PORT,
                    ),
                )
        )
        .get_matches();

    let log_level = match app.value_of("debug") {
        Some(log_level) => {
            match log_level {
                _ if log_level == "0" => log::LevelFilter::Off,
                _ if log_level == "1" => log::LevelFilter::Error,
                _ if log_level == "2" => log::LevelFilter::Warn,
                _ if log_level == "3" => log::LevelFilter::Info,
                _ if log_level == "4" => log::LevelFilter::Debug,
                _ if log_level == "4" => log::LevelFilter::Trace,
                _  => log::LevelFilter::Error,
            }
        }
        None => log::LevelFilter::Error,
    };

    let pidfile = match app.value_of("pidfile") {
        Some(pidfile) => pidfile,
        None => DEFAULT_PIDFILE_PATH,
    };

    let port = match app.value_of("port") {
        Some(port) => port,
        None => DEFAULT_WEB_SERVER_PORT,
    };

    init_logger(
        log_level,
        Some(&PathBuf::from(DEFAULT_LOG_DIR)),
        Some(&PathBuf::from(DEFAULT_LOG_FILE)),
        true,
    ).chain_err(|| ErrorKind::LogError("Unable to initialize logger"))?;

    let port = port.to_owned();

    let data_dir = env::current_dir().unwrap().to_str().unwrap().to_owned();
    let data_dir = data_dir.to_string() + "/www/";
    let data_dir_clone = data_dir.clone();
    spawn(move||web_server(&port, &data_dir_clone));

    main_loop(pidfile, &data_dir)?;
    Ok(())
}

fn main_loop(
    pidfile:    &str,
    data_dir:    &str,
) -> Result<()> {
    let data_file = data_dir.to_string() + "data/nodes.json";
    loop {
        debug!("Start fresh.");
        let data = get_data(pidfile)?;
        write_json(&data_file, data)?;
        debug!("Finnish fresh.");
        sleep(Duration::from_millis(20000));
    }
}

fn get_data(pid_path: &str) -> Result<String> {
    let mut _data_str = String::new();

    loop {
        let mut tinc_stream = TincStream::new(pid_path)
            .chain_err(|| ErrorKind::NotFindTincPiDError("Not find tinc pid file, Run tincd first."))?;

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
        let _ = fs::create_dir_all(path);
        let _ = fs::remove_dir(path);
        let _ = fs::File::create(path);
    }
    fs::write(file_path, data)
        .chain_err(|| ErrorKind::WriteJsonError("when write json to file"))?;
    Ok(())
}
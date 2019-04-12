#[macro_use]
extern crate log;

extern crate serde;
extern crate serde_json;

extern crate tincmgr;
use tincmgr::tinc_tcp_stream::TincStream;
use tincmgr::domain::{Data, TincDump};

use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;
use std::path::Path;
use std::fs;
use std::io::{Result, Error};
use std::io::ErrorKind;

const DEFAULT_PIDFILE_PATH: &str = "/root/tinc/tinc.pid";
const DEFAULT_TINC_FILE_PATH: &str = "/root/tinc/nodes.json";

fn main() {
    match run() {
        Ok(_) => (),
        Err(e) => println!("{:#?}", e),
    };
}

fn run() -> Result<()> {
    let pid_file = DEFAULT_PIDFILE_PATH;
    let tinc_data = get_data(pid_file);
    let file_path = DEFAULT_TINC_FILE_PATH;
    write_json(file_path, tinc_data)?;
    Ok(())
}

fn get_data(pid_path: &str) -> Result<String> {
    let mut _data_str = String::new();
    let now = Instant::now();

    loop {
        let mut tinc_stream = match TincStream::new(pid_path) {
            Ok(x) => x,
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::ConnectionRefused,
                    "Tinc tcp connection refused"
                ))
            }
        };

        if let Ok(nodes) = tinc_stream.dump_nodes() {
            debug!("dump_nodes.");
            if let Ok(edges) = tinc_stream.dump_edges() {
                debug!("dump_edges.");
                if let Ok(subnets) = tinc_stream.dump_subnets() {
                    debug!("dump_subnets.");
                    let data = TincDump::new(
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
        if Instant::now().duration_since(now) > Duration::from_secs(5) {
            return Err(Error::new(ErrorKind::TimedOut, ""));
        }
    }
    Ok(_data_str)
}

fn write_json(file_path: &str, tinc_data: Result<String>) -> Result<()> {
    let path = Path::new(file_path);
    if !path.is_file() {
        let _ = fs::create_dir_all(path);
        let _ = fs::remove_dir(path);
        let _ = fs::File::create(path);
    }

    let mut code: u32 = 200;
    let mut err: Option<String> = None;

    let tinc_data = match tinc_data {
        Ok(x) => Some(x),
        Err(e) => {
            code = 503;
            err = Some(e.to_string());
            None
        },
    };

    let data = Data::new(tinc_data, err, code);
    if let Ok(data) = serde_json::to_string(&data) {
        fs::write(file_path, data)?;
    }
    Ok(())
}
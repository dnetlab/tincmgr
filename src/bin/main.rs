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
const TINC_TCPSTREAM_TIMEOUT_SEC: u64 = 5;

fn main() {
    match run() {
        Ok(_) => (),
        Err(e) => println!("{:#?}", e),
    };
}

/// tinc连接的错误信息传递到 nodes.json
/// 写数据错误, 上报到main()
fn run() -> Result<()> {
    let pid_file = DEFAULT_PIDFILE_PATH;
    let tinc_data = get_data(pid_file);
    let file_path = DEFAULT_TINC_FILE_PATH;
    write_json(file_path, tinc_data)?;
    Ok(())
}

/// 1.读取tinc pid 文件
/// 2.创建tinc control tcp连接
/// 3.获取tinc dump信息
/// 4.解析tinc原信息
fn get_data(pid_path: &str) -> Result<TincDump> {
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
                    if let Ok(_) = tinc_stream.purge() {
                        ()
                    }
                    return Ok(data)
                }
            }
        }
        sleep(Duration::from_millis(1000));
        if Instant::now().duration_since(now)
            > Duration::from_secs(TINC_TCPSTREAM_TIMEOUT_SEC) {
            return Err(Error::new(ErrorKind::TimedOut, ""));
        }
    }
}

/// 把TincDump数据 写到文件
fn write_json(file_path: &str, tinc_data: Result<TincDump>) -> Result<()> {
    // 创建文件地址
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

    let data = serde_json::to_string(&data)
        .map_err(|_| Error::new(
            ErrorKind::InvalidData,
            "Can not parse output data to json"))?;

    fs::write(file_path, data)?;

    Ok(())
}
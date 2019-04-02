use std::io::Result;

use super::tinc_tcp_stream::TincStream;

pub fn stop(pid_path: &str) -> Result<()> {
    let mut tinc_stream = TincStream::new(pid_path)?;
    tinc_stream.stop()?;
    Ok(())
}

pub fn reload(pid_path: &str) -> Result<()> {
    let mut tinc_stream = TincStream::new(pid_path)?;
    tinc_stream.reload()?;
    Ok(())
}

// Tinc can not handle this control.
//pub fn restart(pid_path: &str) -> Result<()> {
//    let mut tinc_stream = TincStream::new(pid_path)?;
//    tinc_stream.restart()?;
//    Ok(())
//}

pub fn dump_nodes(pid_path: &str) -> Result<()> {
    let mut tinc_stream = TincStream::new(pid_path)?;
    tinc_stream.dump_nodes()?;
    Ok(())
}

pub fn dump_subnets(pid_path: &str) -> Result<()> {
    let mut tinc_stream = TincStream::new(pid_path)?;
    tinc_stream.dump_subnets()?;
    Ok(())
}


pub fn dump_connections(pid_path: &str) -> Result<()> {
    let mut tinc_stream = TincStream::new(pid_path)?;
    let _res = tinc_stream.dump_connections()?;
    Ok(())
}

pub fn dump_graph(pid_path: &str) -> Result<()> {
    let mut tinc_stream = TincStream::new(pid_path)?;
    tinc_stream.dump_graph()?;
    Ok(())
}

pub fn purge(pid_path: &str) -> Result<()> {
    let mut tinc_stream = TincStream::new(pid_path)?;
    tinc_stream.purge()?;
    Ok(())
}

pub fn set_debug(pid_path: &str, debug_level: i8) -> Result<()> {
    let mut tinc_stream = TincStream::new(pid_path)?;
    tinc_stream.set_debug(debug_level)?;
    Ok(())
}

pub fn retry(pid_path: &str) -> Result<()> {
    let mut tinc_stream = TincStream::new(pid_path)?;
    tinc_stream.retry()?;
    Ok(())
}

pub fn connect(pid_path: &str) -> Result<()> {
    let mut tinc_stream = TincStream::new(pid_path)?;
    tinc_stream.connect()?;
    Ok(())
}

pub fn disconnect(pid_path: &str) -> Result<()> {
    let mut tinc_stream = TincStream::new(pid_path)?;
    tinc_stream.disconnect()?;
    Ok(())
}

pub fn dump_traffic(pid_path: &str) -> Result<()> {
    let mut tinc_stream = TincStream::new(pid_path)?;
    tinc_stream.dump_traffic()?;
    Ok(())
}

pub fn pcap(pid_path: &str) -> Result<()> {
    let mut tinc_stream = TincStream::new(pid_path)?;
    tinc_stream.pcap()?;
    Ok(())
}

pub fn log(pid_path: &str) -> Result<()> {
    let mut tinc_stream = TincStream::new(pid_path)?;
    tinc_stream.log()?;
    Ok(())
}
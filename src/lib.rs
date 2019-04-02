extern crate chrono;
#[macro_use]
extern crate error_chain;
extern crate fern;
#[macro_use]
extern crate serde;

#[macro_use]
extern crate log;

extern crate derive_try_from_primitive;

pub mod tinc_tcp_stream;
pub mod control;
pub mod logging;
pub mod domain;
pub mod web_server;
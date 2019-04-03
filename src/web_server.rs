extern crate actix_web;

use actix_web::fs::StaticFiles;
use actix_web::{middleware, App, server::HttpServer};

pub fn web_server(port: &str, data_dir: &str) -> std::io::Result<()> {
    let data_dir = data_dir.to_string();

    HttpServer::new(move|| {
        App::new()
            .middleware(middleware::Logger::default())
            .handler("/",
                     StaticFiles::new(
                         &data_dir)
                         .expect("Web server load static files failed.")
                         .index_file("index.html"),
            )
    })
        .bind("0.0.0.0:".to_string() + port)?
        .run();
    Ok(())
}
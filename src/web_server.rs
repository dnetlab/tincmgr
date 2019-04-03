use actix_files;
use actix_web::{middleware, App, HttpServer};

pub fn web_server(port: &str, data_dir: &str) -> std::io::Result<()> {
    let data_dir = data_dir.to_string();
    HttpServer::new(move || {
        let data_dir_clone = data_dir.clone();
        App::new()
            .wrap(middleware::Logger::default())
            .service(
                actix_files::Files::new("/", data_dir_clone).index_file("index.html"),
            )
    })
        .bind("0.0.0.0:".to_string() + port)?
        .run()
}
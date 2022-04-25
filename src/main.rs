use actix_files::Files;
use actix_web::{
    web::{get, Data},
    App, HttpServer,
};
use libreads::{libreads::LibReads, web::download};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let libreads = Data::new(LibReads::default());

    HttpServer::new(move || {
        App::new()
            .service(Files::new("/", "./frontend/build").index_file("index.html"))
            .route("/download/{goodreads_url}", get().to(download))
            .app_data(libreads.clone())
    })
    .bind(("127.0.0.1", 8001))?
    .run()
    .await
}

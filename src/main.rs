use actix_web::{web::Data, App, HttpServer};
use libreads::{libreads::LibReads, web::download};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let libreads = Data::new(LibReads::default());

    HttpServer::new(move || {
        App::new()
            .route(
                "/download/{goodreads_url}",
                actix_web::web::get().to(download),
            )
            .app_data(libreads.clone())
    })
    .bind(("127.0.0.1", 8001))?
    .run()
    .await
}

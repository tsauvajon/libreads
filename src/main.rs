use actix_web::{App, HttpServer};
use libreads::web::download;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().route(
            "/download/{goodreads_url}",
            actix_web::web::get().to(download),
        )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

use axum::{Router, routing::post};
use hyper::Server; // Importe `Server` do `hyper`

use std::net::SocketAddr;

mod routes;
mod converters;

#[tokio::main]
async fn main() {
    // Configura a rota `/convert` para aceitar POST requests
    let app = Router::new().route("/convert", post(routes::convert::convert_file));

    // Configura o endereço do servidor
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Servidor rodando em http://{}", addr);

    // Inicia o servidor
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

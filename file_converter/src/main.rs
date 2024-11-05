use axum::{Router, routing::post};
use std::net::SocketAddr;

mod routes; // Importa o módulo de rotas
mod converters; // Importa o módulo de conversores

#[tokio::main]
async fn main() {
    // Criação do aplicativo com a rota /convert
    let app = Router::new().route("/convert", post(routes::convert::convert_file));

    // Definindo o endereço do servidor
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Servidor rodando em http://{}", addr);

    // Inicia o servidor usando `hyper`
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

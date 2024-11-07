use axum::{extract::Json, response::IntoResponse, routing::post};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ConvertRequest {
    input_file: String,
    format: String, // Formato para o qual o arquivo será convertido
}

pub async fn convert_file(Json(payload): Json<ConvertRequest>) -> impl IntoResponse {
    // Aqui você adicionaria a lógica para converter arquivos
    // Esta é uma resposta de exemplo
    let response_message = format!("Convertendo arquivo: {} para o formato: {}", payload.input_file, payload.format);
    (axum::http::StatusCode::OK, response_message) // Retorna o status e a mensagem
}

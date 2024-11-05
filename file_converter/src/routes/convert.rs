use axum::{Json, extract::Multipart};
use serde_json::json;
use crate::converters::{csv_to_json::convert_csv_to_json, text_to_md::convert_text_to_md};

/// Handler para a rota `/convert`, processa o upload e chama as funções de conversão apropriadas.
pub async fn convert_file(mut multipart: Multipart) -> Json<serde_json::Value> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let file_name = field.file_name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        // Escolhe a conversão com base na extensão do arquivo
        if file_name.ends_with(".csv") {
            let json_data = convert_csv_to_json(&data);
            return Json(json!({"message": "Arquivo CSV convertido para JSON", "data": json_data}));
        } else if file_name.ends_with(".txt") {
            let md_data = convert_text_to_md(&data);
            return Json(json!({"message": "Arquivo TXT convertido para Markdown", "data": md_data}));
        }
    }

    Json(json!({"error": "Formato de arquivo não suportado"}))
}

use actix_web::{post, App, HttpResponse, HttpServer, Responder};
use csv::ReaderBuilder;
use pulldown_cmark::{html, Parser};
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::io::Cursor;

#[derive(Serialize)]
struct CSVRow(HashMap<String, String>);

#[post("/csv_to_json")]
async fn csv_to_json(csv_data: String) -> impl Responder {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(Cursor::new(csv_data));

    // Armazena os cabeçalhos antes do loop
    let headers = match reader.headers() {
        Ok(headers) => headers.clone(),
        Err(_) => return HttpResponse::BadRequest().body("Erro ao processar CSV."),
    };

    let mut rows = Vec::new();

    for result in reader.records() {
        match result {
            Ok(record) => {
                let mut row = HashMap::new();
                for (header, value) in headers.iter().zip(record.iter()) {
                    row.insert(header.to_string(), value.to_string());
                }
                rows.push(CSVRow(row));
            }
            Err(_) => return HttpResponse::BadRequest().body("Erro ao processar CSV."),
        }
    }

    HttpResponse::Ok().json(json!(rows))
}

#[post("/text_to_markdown")]
async fn text_to_markdown(text: String) -> impl Responder {
    let parser = Parser::new(&text);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    HttpResponse::Ok().body(html_output)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(csv_to_json)
            .service(text_to_markdown)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

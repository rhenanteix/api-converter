use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use csv::ReaderBuilder;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::io::Cursor;

#[derive(Deserialize)]
struct CsvData {
    csvData: String,
}

#[derive(Serialize)]
struct CSVRow(HashMap<String, String>);

#[get("/")]
async fn hello_world() -> &'static str {
    "Api em Rust rodando"
}

#[post("/convert/csv-to-json")]
async fn csv_to_json(body: web::Json<CsvData>) -> impl Responder {
    let csv_data = &body.csvData;

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(Cursor::new(csv_data));

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

    let json_data = json!(rows).to_string();

    HttpResponse::Ok()
        .insert_header(("Content-Disposition", "attachment; filename=output.json"))
        .content_type("application/json")
        .body(json_data)
}

#[post("/convert/csv_to_sql")]
async fn csv_to_sql(body: web::Json<CsvData>) -> impl Responder {
    let csv_data = &body.csvData;

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(Cursor::new(csv_data));

    let headers = match reader.headers() {
        Ok(headers) => headers.clone(),
        Err(_) => return HttpResponse::BadRequest().body("Erro ao processar CSV."),
    };

    let mut sql_statements = Vec::new();
    
    for result in reader.records() {
        match result {
            Ok(record) => {
                let values: Vec<String> = record.iter()
                    .map(|value| format!("'{}'", value))
                    .collect();

                let sql = format!(
                    "INSERT INTO tabela ({}) VALUES ({});",
                    headers.iter().collect::<Vec<&str>>().join(", "),
                    values.join(", ")
                );
                sql_statements.push(sql);
            }
            Err(_) => return HttpResponse::BadRequest().body("Erro ao processar CSV."),
        }
    }

    let sql_data = sql_statements.join("\n");

    HttpResponse::Ok()
        .insert_header(("Content-Disposition", "attachment; filename=output.sql"))
        .content_type("application/sql")
        .body(sql_data)
}

#[post("/convert/json-to-yaml")]
async fn json_to_yaml(body: web::Json<Value>) -> impl Responder {
    match serde_yaml::to_string(&body) {
        Ok(yaml_data) => {
            HttpResponse::Ok()
                .insert_header(("Content-Disposition", "attachment; filename=output.yaml"))
                .content_type("application/x-yaml")
                .body(yaml_data)
        }
        Err(_) => HttpResponse::BadRequest().body("Erro ao processar JSON para YAML."),
    }
}




#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(cors)
            .service(hello_world)
            .service(csv_to_json)
            .service(csv_to_sql)
            .service(json_to_yaml)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

use actix_web::{post, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use csv::ReaderBuilder;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::io::Cursor;




#[derive(Serialize)]
struct CSVRow(HashMap<String, String>);

#[post("/csv_to_sql")]
async fn csv_to_sql(csv_data: String) -> impl Responder {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(Cursor::new(csv_data));

    // Lê os headers do CSV
    let headers = match reader.headers() {
        Ok(headers) => headers.clone(),
        Err(_) => return HttpResponse::BadRequest().body("Erro ao processar CSV."),
    };

    let mut sql_statements = Vec::new();
    
    // Vamos gerar a instrução SQL de inserção baseada nas colunas do CSV
    for result in reader.records() {
        match result {
            Ok(record) => {
                // Convertendo StringRecord em um Vec<String>
                let values: Vec<String> = record.iter()
                    .map(|value| format!("'{}'", value)) // Adiciona aspas para os valores
                    .collect();

                // Gera a instrução SQL
                let sql = format!(
                    "INSERT INTO tabela ({}) VALUES ({});",
                    headers.iter().collect::<Vec<&str>>().join(", "), // Usando join no vetor de headers
                    values.join(", ")
                );
                sql_statements.push(sql);
            }
            Err(_) => return HttpResponse::BadRequest().body("Erro ao processar CSV."),
        }
    }

    let sql_data = sql_statements.join("\n");

    // Retornar o arquivo SQL como resposta
    HttpResponse::Ok()
        .insert_header(("Content-Disposition", "attachment; filename=output.sql"))
        .content_type("application/sql")
        .body(sql_data)
}


#[post("/csv_to_json")]
async fn csv_to_json(csv_data: String) -> impl Responder {
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

    // Serializar os dados JSON em uma string
    let json_data = json!(rows).to_string();

    // Retornar o JSON como um arquivo para download
    HttpResponse::Ok()
        .insert_header(("Content-Disposition", "attachment; filename=output.json"))
        .content_type("application/json")
        .body(json_data)
}

#[post("/json_to_yaml")]
async fn json_to_yaml(json_data: String) -> impl Responder {
    match serde_json::from_str::<serde_json::Value>(&json_data) {
        Ok(parsed_json) => {
            // Converter JSON para YAML
            match serde_yaml::to_string(&parsed_json) {
                Ok(yaml) => HttpResponse::Ok().body(yaml),
                Err(_) => HttpResponse::BadRequest().body("Erro ao converter JSON para YAML."),
            }
        }
        Err(_) => HttpResponse::BadRequest().body("Erro ao processar JSON."),
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
            .service(csv_to_json)
            .service(csv_to_sql)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

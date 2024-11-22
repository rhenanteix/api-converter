use actix_web::{get, error, post, web, App, HttpResponse, HttpServer, Responder, Error};
use actix_multipart::Multipart;
use actix_cors::Cors;
use arrow::csv::{reader, writer};
use arrow::datatypes::Schema;
use csv::ReaderBuilder;
use parquet::data_type::ByteArray;
use parquet::file::writer::{SerializedFileWriter};
use parquet::record;
use parquet::schema::parser::parse_message_type;
use serde::{Deserialize, Serialize};
use serde_json::{from_reader, json};
use serde_json::Value;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;
use reqwest::Client;
use futures_util::StreamExt as _; // Para lidar com o fluxo de dados multipart
use std::fs::File;
use apache_avro::{Schema as Sch, Writer};
use parquet::file::properties::WriterProperties;


#[derive(Deserialize)]
struct CsvData {
    csvData: String,
    webhook_url: Option<String>,
    csvFiles: Option<Vec<String>>, // Torna o campo opcional
}

#[derive(Serialize)]
struct WebHookPayload {
    status: String,
    message: String,
    download_link: Option<String>
}

#[derive(Serialize)]
struct CSVRow(HashMap<String, String>);

#[get("/convert/service")]
async fn hello_world() -> &'static str {
    "API EM RUST RODANDO EM"
}

#[post("/convert/csv-to-json/batch")]
async fn csv_to_json_batch(mut payload: Multipart) -> impl Responder { 
    let mut results = Vec::new();

    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(_) => return HttpResponse::BadRequest().body("Error processing file."),
        };

        let mut csv_data = Vec::new();

        while let Some(chunk) = field.next().await {
            match chunk {
                Ok(data) => csv_data.extend(data),
                Err(_) => return HttpResponse::BadRequest().body("Error processing file."),
            }
        }

        // Convert CSV to JSON
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(&csv_data[..]);

        let mut rows = Vec::new();
        for result in reader.records() {
            match result {
                Ok(record) => {
                    let row: Vec<String> = record.iter().map(|v| v.to_string()).collect();
                    rows.push(json!(row));
                }
                Err(err) => {
                    println!("Erro ao processar o registro CSV: {:?}", err);  // Log de erro detalhado
                    return HttpResponse::BadRequest().body("Error converting file.");
                }
            }
        }

        // Store result for this file
        results.push(json!({ "file": rows }));
    }

    // Return all results as a JSON response
    HttpResponse::Ok().json(results)
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

    let download_link = "https://meusite.com/download/output.json".to_string();

    if let Some(webhook_url) = &body.webhook_url  {
        let client = Client::new();
        let payload = WebHookPayload {
            status: "concluido".to_string(),
            message: "Conversão para JSON concluída com sucesso.".to_string(),
            download_link: Some(download_link.clone()),
        };

        let _ = client
            .post(webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|err| println!("Erro ao enviar webhook: {:?}", err));
    }

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

#[post("/convert/csv-to-parquet")]
async fn csv_to_parquet(body: web::Json<CsvData>) -> Result<HttpResponse, actix_web::Error> {
    let csv_data = &body.csvData;

    // Ler o CSV
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(Cursor::new(csv_data));

    // Obter os cabeçalhos do CSV
    let headers = match reader.headers() {
        Ok(headers) => headers.clone(),
        Err(_) => return Ok(HttpResponse::BadRequest().body("Erro ao processar CSV.")),
    };
    
    // Construir o esquema Parquet dinamicamente com base nos cabeçalhos do CSV
    let schema_str = format!(
        "message schema {{ {} }}",
        headers
            .iter()
            .map(|col| format!("required binary {} (UTF8);", col))
            .collect::<Vec<String>>()
            .join(" ")
    );

    let parsed_schema = match parse_message_type(&schema_str) {
        Ok(schema) => Arc::new(schema),
        Err(_) => return Ok(HttpResponse::InternalServerError().body("Erro ao criar esquema Parquet.")),
    };

    // Criar propriedades do escritor
    let writer_props = Arc::new(WriterProperties::builder().build());

    // Criar o arquivo Parquet na memória
    let mut buffer = Vec::new();
    let mut writer = SerializedFileWriter::new(&mut buffer, parsed_schema, writer_props).unwrap();

    // Criar um RowGroupWriter para escrever os registros
    let mut row_group_writer = writer.next_row_group().unwrap();
    
    for result in reader.records() {
        match result {
            Ok(record) => {
                if record.len() != headers.len() {
                    return Err(error::ErrorBadRequest("Número de colunas no CSV não corresponde ao esquema."));
                }
    
                for (i, value) in record.iter().enumerate() {
                    if let Some(mut col_writer) = row_group_writer
                        .next_column()
                        .map_err(|e| error::ErrorInternalServerError(format!("Erro ao obter próxima coluna: {}", e)))? 
                    {
                        let sanitized_value = value.replace('\n', "").replace('\r', "");
                        let value_as_byte_array = ByteArray::from(sanitized_value.as_bytes());
                        col_writer
                            .typed::<parquet::data_type::ByteArrayType>()
                            .write_batch(&[value_as_byte_array], None, None)
                            .map_err(|e| error::ErrorInternalServerError(format!("Erro ao escrever no Parquet: {}", e)))?;
                        col_writer.close().unwrap();
                    }
                }
            }
            Err(e) => {
                return Err(error::ErrorBadRequest(format!("Erro ao processar registro CSV: {}", e)));
            }
        }
    }
    

// Fechar RowGroupWriter após todas as colunas serem processadas
row_group_writer.close().unwrap();

    writer.close().unwrap();

    // Retornar o arquivo Parquet como resposta
    Ok(HttpResponse::Ok()
    .insert_header(("Content-Disposition", "attachment; filename=output.parquet"))
    .content_type("application/parquet")
    .body(buffer))

}



#[post("/convert/csv-to-avro")]
async fn csv_to_avro(body: web::Json<CsvData>) -> impl Responder {
    let csv_data = &body.csvData;

    // Criar leitor de CSV
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(Cursor::new(csv_data));

    // Obter cabeçalhos do CSV
    let headers = match reader.headers() {
        Ok(headers) => headers.clone(),
        Err(_) => return HttpResponse::BadRequest().body("Erro ao processar CSV."),
    };

    // Definir o esquema Avro
    let schema_str = r#"
    {
        "type": "record",
        "name": "CSVRecord",
        "fields": [
            {"name": "column1", "type": "string"},
            {"name": "column2", "type": "string"}
        ]
    }
    "#;

    // Parse do esquema Avro
    let schema = match Sch::parse_str(schema_str) {
        Ok(schema) => schema,
        Err(_) => return HttpResponse::InternalServerError().body("Erro ao criar esquema Avro."),
    };

    // Criar o escritor Avro
    let mut writer = Writer::new(&schema, Vec::new());

    // Processar registros do CSV e adicionar ao escritor Avro
    for result in reader.records() {
        match result {
            Ok(record) => {
                let mut avro_row = HashMap::new();
                for (header, value) in headers.iter().zip(record.iter()) {
                    avro_row.insert(header.to_string(), Value::String(value.to_string()));
                }
                if let Err(e) = writer.append_ser(avro_row) {
                    return HttpResponse::InternalServerError().body(format!("Erro ao escrever Avro: {}", e));
                }
            }
            Err(_) => return HttpResponse::BadRequest().body("Erro ao processar CSV."),
        }
    }

    // Finalizar e obter os dados Avro
    let avro_data = match writer.into_inner() {
        Ok(data) => data,
        Err(_) => return HttpResponse::InternalServerError().body("Erro ao finalizar escrita Avro."),
    };

    // Retornar arquivo Avro como resposta
    HttpResponse::Ok()
        .insert_header(("Content-Disposition", "attachment; filename=output.avro"))
        .content_type("application/avro")
        .body(avro_data)
}



// Exemplo de rotas
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
            .service(csv_to_json_batch)
            .service(csv_to_avro)
            .service(csv_to_parquet)
    })
    .bind("0.0.0.0:8080")? // Alterado para 0.0.0.0 para garantir que o serviço esteja disponível externamente
    .run()
    .await
}


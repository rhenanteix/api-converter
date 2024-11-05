use serde_json::Value;
use csv::ReaderBuilder;
use std::io::Cursor;

/// Converte um arquivo CSV em um JSON.
pub fn convert_csv_to_json(data: &[u8]) -> Value {
    let mut rdr = ReaderBuilder::new().from_reader(Cursor::new(data));
    let headers = rdr.headers().unwrap().clone();

    let mut records = vec![];

    for result in rdr.records() {
        let record = result.unwrap();
        let mut json_record = serde_json::Map::new();

        for (header, value) in headers.iter().zip(record.iter()) {
            json_record.insert(header.to_string(), Value::String(value.to_string()));
        }
        records.push(Value::Object(json_record));
    }

    Value::Array(records)
}

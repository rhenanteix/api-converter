use std::str;

/// Converte um texto simples em formato Markdown.
pub fn convert_text_to_md(data: &[u8]) -> String {
    let text = str::from_utf8(data).unwrap_or("");
    format!("# Texto Convertido para Markdown\n\n{}", text)
}

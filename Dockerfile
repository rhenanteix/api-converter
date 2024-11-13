# Usa a imagem oficial do Rust como base
FROM rust:latest

# Define o diretório de trabalho dentro do container
WORKDIR /app

# Copia o arquivo Cargo.toml e Cargo.lock para o diretório de trabalho
COPY Cargo.toml ./

# Baixa as dependências, permitindo cache para acelerar o build
RUN cargo fetch

# Copia o código-fonte do projeto para o diretório de trabalho
COPY . .

# Compila o projeto em modo release
RUN cargo build --release

# Expõe a porta (se necessário)
EXPOSE 3000

# Executa o binário da aplicação
CMD ["./target/release/api-converter"]

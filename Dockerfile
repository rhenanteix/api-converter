# Usa a imagem oficial do Rust como base
FROM rust:latest AS builder

# Define o diretório de trabalho dentro do container
WORKDIR /app

# Copia Cargo.toml e Cargo.lock para aproveitar o cache Docker ao instalar dependências
COPY Cargo.toml Cargo.lock ./

# Instala dependências
RUN cargo build --release || true

# Copia o restante do código fonte
COPY . .

# Compila o projeto em modo release
RUN cargo build --release

# Imagem final, mais leve
FROM debian:buster-slim

# Copia o binário do Rust da etapa de construção
COPY --from=builder /app/target/release/api-converter /usr/local/bin/api-converter

# Expõe a porta necessária
EXPOSE 3000

# Define o comando de inicialização
CMD ["api-converter"]

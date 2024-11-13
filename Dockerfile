# Etapa 1: Construção
FROM rust:latest AS builder

WORKDIR /app

# Copia apenas o Cargo.toml e o Cargo.lock, caso exista, para baixar as dependências primeiro
COPY Cargo.toml Cargo.lock ./

# Baixa as dependências e as mantém em cache, evitando downloads repetidos
RUN cargo fetch

# Agora copia o código fonte e compila
COPY . .
RUN cargo build --release

# Etapa 2: Imagem Final
FROM debian:buster-slim

# Copia o binário do Rust da etapa de construção
COPY --from=builder /app/target/release/api-converter /usr/local/bin/api-converter

# Expõe a porta necessária
EXPOSE 3000

# Define o comando de inicialização
CMD ["api-converter"]

# Etapa de construção com musl
FROM rust:latest AS builder

WORKDIR /app

# Copia o Cargo.toml e Cargo.lock
COPY Cargo.toml Cargo.lock ./

# Baixa as dependências para cache
RUN cargo fetch

# Copia o restante do código fonte
COPY . .

# Compila com musl para um binário estático
RUN cargo build --release --target x86_64-unknown-linux-musl

# Imagem final, menor e sem dependência de glibc
FROM alpine:latest

# Copia o binário da etapa de build
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/api-converter /usr/local/bin/api-converter

# Expõe a porta necessária
EXPOSE 3000

# Executa o binário da aplicação
CMD ["api-converter"]

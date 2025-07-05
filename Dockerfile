# Etapa 1: build da aplicação
FROM rust:latest as builder

# Cria diretório de trabalho
WORKDIR /app

# Copia os arquivos da aplicação
COPY . .

# Compila o projeto em modo release
RUN cargo build --release

# Etapa 2: imagem final
FROM ubuntu:22.04

# Instala dependências básicas
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
 && rm -rf /var/lib/apt/lists/*
# Cria diretório para a aplicação
WORKDIR /app

# Copia o binário da aplicação compilado
COPY --from=builder /app/target/release/GE-upload-file-api ./api

# Expõe a porta da API (ajuste conforme necessário)
EXPOSE 3000

# Comando para iniciar a API
CMD ["./api"]


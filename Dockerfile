# Stage 1: Build (Estágio de Compilação)
FROM rust:1.86-slim AS builder

WORKDIR /usr/src/app

# Instala ferramentas necessárias para build (incluindo SQLite e OpenSSL se necessário)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

# Copia as dependências e o código fonte
COPY . .

# Compila a aplicação para produção (Release)
RUN cargo build --release

# Stage 2: Runtime (Estágio de Execução da Imagem Final)
FROM debian:bookworm-slim

WORKDIR /app

# Instala pacotes mínimos de segurança de rede (ca-certificates)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Cria diretório para persistência do banco SQLite
RUN mkdir -p /app/data

# Copia o binário compilado no estágio anterior
COPY --from=builder /usr/src/app/target/release/rustavoid /app/rustavoid
COPY --from=builder /usr/src/app/migrations /app/migrations
COPY --from=builder /usr/src/app/static /app/static

# Define as variáveis de ambiente padrões do container
ENV PORT=8080
ENV RUST_LOG=rustavoid=info,tower_http=info
ENV DATABASE_URL=sqlite:///app/data/rustavoid.db?mode=rwc

# Expõe a porta configurada
EXPOSE 8080

# Executa o binário do app
CMD ["./rustavoid"]

# Multi-stage build para otimizar o tamanho da imagem final
FROM rust:1.87-slim as builder

# Instalar dependências do sistema necessárias para compilar
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Definir diretório de trabalho
WORKDIR /app

# Copiar arquivos de dependências primeiro (para aproveitar cache do Docker)
COPY Cargo.toml Cargo.lock ./

# Criar um arquivo lib.rs vazio para compilar dependências
RUN mkdir -p src && echo "fn main() {}" > src/lib.rs

# Compilar dependências (isso será cacheado se Cargo.toml não mudar)
RUN cargo build --release

# Remover o arquivo lib.rs temporário
RUN rm src/lib.rs

# Copiar o código fonte
COPY src/ ./src/
COPY migrations/ ./migrations/

# Compilar a aplicação
RUN cargo build --release

# Imagem final otimizada
FROM debian:bookworm-slim

# Instalar runtime dependencies e curl para healthcheck
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Criar usuário não-root para segurança
RUN groupadd -r rustapp && useradd -r -g rustapp rustapp

# Definir diretório de trabalho
WORKDIR /app

# Copiar binário compilado da etapa anterior
COPY --from=builder /app/target/release/rust_template ./rust_template

# Copiar migrations
COPY --from=builder /app/migrations ./migrations

# Definir variáveis de ambiente padrão
ENV APP_ENVIRONMENT=production
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080
ENV DATABASE_MAX_CONNECTIONS=10
ENV JWT_EXPIRES_IN=86400

# Mudar propriedade dos arquivos para o usuário rustapp
RUN chown -R rustapp:rustapp /app

# Mudar para usuário não-root
USER rustapp

# Expor porta
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/v1/health/ || exit 1

# Comando para executar a aplicação
CMD ["./rust_template"] 
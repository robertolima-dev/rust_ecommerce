#!/bin/bash

# Script para executar testes com banco de teste

echo "🚀 Iniciando testes do Rust Template..."

# Verificar se DATABASE_URL_TEST está configurada
if [ -z "$DATABASE_URL_TEST" ]; then
    echo "❌ Erro: DATABASE_URL_TEST não está configurada"
    echo "Por favor, configure a variável DATABASE_URL_TEST no seu .env"
    echo "Exemplo: DATABASE_URL_TEST=postgresql://user:pass@localhost:5432/rust_template_test"
    exit 1
fi

# Configurar ambiente para testes
export APP_ENVIRONMENT=testing
export RUST_LOG=info

echo "📊 Configurações de teste:"
echo "  - Ambiente: $APP_ENVIRONMENT"
echo "  - Banco de teste: $DATABASE_URL_TEST"
echo ""

# Executar migrações no banco de teste
echo "🔄 Executando migrações no banco de teste..."
sqlx database create --database-url "$DATABASE_URL_TEST" 2>/dev/null || true
sqlx migrate run --database-url "$DATABASE_URL_TEST"

if [ $? -ne 0 ]; then
    echo "❌ Erro ao executar migrações no banco de teste"
    exit 1
fi

echo "✅ Migrações executadas com sucesso"
echo ""

# Executar testes
echo "🧪 Executando testes..."
cargo test --test user_tests -- --nocapture

# Capturar o resultado dos testes
TEST_RESULT=$?

echo ""
echo "🧹 Limpando banco de teste..."

# Remover banco de teste (opcional - descomente se quiser)
# sqlx database drop --database-url "$DATABASE_URL_TEST" --yes

echo ""
if [ $TEST_RESULT -eq 0 ]; then
    echo "✅ Todos os testes passaram!"
    exit 0
else
    echo "❌ Alguns testes falharam"
    exit 1
fi 
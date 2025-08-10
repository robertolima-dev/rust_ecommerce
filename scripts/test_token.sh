#!/bin/bash

set -e

# Script simples para testar token do Keycloak
# Uso: ./test_token.sh

echo "🔑 Obtendo token do Keycloak..."
RESPONSE=$(./get_keycloak_token.sh)
TOKEN=$(echo "$RESPONSE" | awk '/Access Token:/ {getline; print $0}' | tr -d '\n')

echo "🔐 Token: $TOKEN"

echo "🧪 Testando endpoint da aplicação..."
RESPONSE_API=$(curl -s -X POST http://localhost:8080/api/v1/auth/login-keycloak/ \
  -H "Content-Type: application/json" \
  -d '{"provider_token": "'$TOKEN'"}')

echo "📡 Resposta da aplicação:"
echo "$RESPONSE_API" | jq . || echo "$RESPONSE_API"

echo "\n🎉 Teste concluído!" 
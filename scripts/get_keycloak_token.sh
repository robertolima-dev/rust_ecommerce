#!/bin/bash

# Script para obter token do Keycloak para testes
# Uso: ./get_keycloak_token.sh

# ===== CONFIGURAÇÕES DO KEYCLOAK =====
KEYCLOAK_URL="http://localhost:8082"
REALM="rust-template"
CLIENT_ID="rust-template-client"
CLIENT_SECRET="rust-template-secret-123"  # Mesmo secret do setup_keycloak.sh

# ===== CONFIGURAÇÕES DO USUÁRIO DE TESTE =====
# Altere estas variáveis para usar diferentes usuários de teste
# IMPORTANTE: Use os mesmos valores do setup_keycloak.sh
USERNAME="test-novo4@example.com"
PASSWORD="test123"

echo "🔑 Obtendo token do Keycloak..."
echo "👤 Usuário: ${USERNAME}"
echo ""

# 1. Obter access token
echo "📡 Fazendo login no Keycloak..."

TOKEN_RESPONSE=$(curl -s -X POST \
  "${KEYCLOAK_URL}/realms/${REALM}/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=password" \
  -d "client_id=${CLIENT_ID}" \
  -d "client_secret=${CLIENT_SECRET}" \
  -d "username=${USERNAME}" \
  -d "password=${PASSWORD}")

# Verificar se a resposta contém erro
if echo "$TOKEN_RESPONSE" | grep -q "error"; then
    echo "❌ Erro ao obter token:"
    echo "$TOKEN_RESPONSE" | jq '.'
    exit 1
fi

# Extrair access token
ACCESS_TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.access_token')

if [ "$ACCESS_TOKEN" = "null" ] || [ -z "$ACCESS_TOKEN" ]; then
    echo "❌ Não foi possível extrair o access token"
    echo "Resposta completa:"
    echo "$TOKEN_RESPONSE" | jq '.'
    exit 1
fi

echo "✅ Token obtido com sucesso!"
echo ""
echo "🔐 Access Token:"
echo "$ACCESS_TOKEN"
echo ""
echo "📋 Para testar o endpoint:"
echo "curl -X POST http://localhost:8080/api/v1/auth/login-keycloak/ \\"
echo "  -H \"Content-Type: application/json\" \\"
echo "  -d '{\"provider_token\": \"$ACCESS_TOKEN\"}'"
echo ""
echo ""
echo "🔍 Para decodificar o token, use:"
echo "./decode_token.sh '$ACCESS_TOKEN'" 
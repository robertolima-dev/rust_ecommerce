#!/bin/bash

# Script para configurar senha para usuários reais no Keycloak
# Uso: ./set_user_password.sh <email> <senha>

# ===== CONFIGURAÇÕES DO KEYCLOAK =====
KEYCLOAK_URL="http://localhost:8082"
ADMIN_USERNAME="admin"
ADMIN_PASSWORD="admin"
REALM="rust-template"

# Verificar argumentos
if [ $# -ne 2 ]; then
    echo "❌ Uso: $0 <email> <senha>"
    echo "Exemplo: $0 robertolima.izphera+user12@gmail.com minha_senha123"
    exit 1
fi

USER_EMAIL="$1"
USER_PASSWORD="$2"

echo "🔑 Configurando senha para usuário: ${USER_EMAIL}"
echo ""

# 1. Obter admin token
echo "🔑 Obtendo token de admin..."

ADMIN_TOKEN_RESPONSE=$(curl -s -X POST \
  "${KEYCLOAK_URL}/realms/master/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=password" \
  -d "client_id=admin-cli" \
  -d "username=${ADMIN_USERNAME}" \
  -d "password=${ADMIN_PASSWORD}")

ADMIN_TOKEN=$(echo "$ADMIN_TOKEN_RESPONSE" | jq -r '.access_token')

if [ "$ADMIN_TOKEN" = "null" ] || [ -z "$ADMIN_TOKEN" ]; then
    echo "❌ Erro ao obter token de admin"
    echo "$ADMIN_TOKEN_RESPONSE" | jq '.'
    exit 1
fi

echo "✅ Token de admin obtido!"

# 2. Buscar usuário por email
echo "🔍 Buscando usuário: ${USER_EMAIL}"

# URL encode o email para caracteres especiais
ENCODED_EMAIL=$(echo "$USER_EMAIL" | sed 's/+/%2B/g' | sed 's/@/%40/g')

USER_RESPONSE=$(curl -s -X GET \
  "${KEYCLOAK_URL}/admin/realms/${REALM}/users?email=${ENCODED_EMAIL}" \
  -H "Authorization: Bearer ${ADMIN_TOKEN}")

USER_ID=$(echo "$USER_RESPONSE" | jq -r '.[0].id')

if [ "$USER_ID" = "null" ] || [ -z "$USER_ID" ]; then
    echo "❌ Usuário não encontrado: ${USER_EMAIL}"
    exit 1
fi

echo "✅ Usuário encontrado! ID: ${USER_ID}"

# 3. Configurar senha
echo "🔐 Configurando senha..."

PASSWORD_RESPONSE=$(curl -s -X PUT \
  "${KEYCLOAK_URL}/admin/realms/${REALM}/users/${USER_ID}/reset-password" \
  -H "Authorization: Bearer ${ADMIN_TOKEN}" \
  -H "Content-Type: application/json" \
  -d "{
    \"type\": \"password\",
    \"value\": \"${USER_PASSWORD}\",
    \"temporary\": false
  }")

if [ $? -eq 0 ]; then
    echo "✅ Senha configurada com sucesso!"
    echo ""
    echo "👤 Usuário: ${USER_EMAIL}"
    echo "🔑 Senha: ${USER_PASSWORD}"
    echo ""
    echo "🔑 Para testar, execute:"
    echo "curl -X POST \"${KEYCLOAK_URL}/realms/${REALM}/protocol/openid-connect/token\" \\"
    echo "  -H \"Content-Type: application/x-www-form-urlencoded\" \\"
    echo "  -d \"grant_type=password\" \\"
    echo "  -d \"client_id=rust-template-client\" \\"
    echo "  -d \"client_secret=rust-template-secret-123\" \\"
    echo "  -d \"username=${USER_EMAIL}\" \\"
    echo "  -d \"password=${USER_PASSWORD}\""
else
    echo "❌ Erro ao configurar senha"
    echo "$PASSWORD_RESPONSE"
    exit 1
fi 
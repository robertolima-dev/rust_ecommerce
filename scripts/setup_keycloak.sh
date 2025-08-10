#!/bin/bash

# Script para configurar Keycloak automaticamente
# Uso: ./setup_keycloak.sh

# ===== CONFIGURAÇÕES DO KEYCLOAK =====
KEYCLOAK_URL="http://localhost:8082"
ADMIN_USERNAME="admin"
ADMIN_PASSWORD="admin"
REALM="rust-template"
CLIENT_ID="rust-template-client"
CLIENT_SECRET="rust-template-secret-123"

# ===== CONFIGURAÇÕES DO USUÁRIO DE TESTE =====
# Altere estas variáveis para criar diferentes usuários de teste
TEST_USER_EMAIL="test-novo4@example.com"
TEST_USER_PASSWORD="test123"
TEST_USER_FIRST_NAME="TestNovo4"
TEST_USER_LAST_NAME="User"
TEST_USER_ROLE="user"  # Pode ser: user, admin, super_admin

echo "🚀 Configurando Keycloak..."
echo "👤 Usuário de teste: ${TEST_USER_EMAIL}"
echo "🔑 Senha: ${TEST_USER_PASSWORD}"
echo "👥 Role: ${TEST_USER_ROLE}"
echo ""

# Aguardar Keycloak estar pronto
echo "⏳ Aguardando Keycloak estar pronto..."
until curl -s "${KEYCLOAK_URL}/health" > /dev/null 2>&1; do
    echo "   Aguardando Keycloak..."
    sleep 5
done

echo "✅ Keycloak está pronto!"

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

# 2. Criar realm
echo "🏗️  Criando realm '${REALM}'..."

REALM_RESPONSE=$(curl -s -X POST \
  "${KEYCLOAK_URL}/admin/realms" \
  -H "Authorization: Bearer ${ADMIN_TOKEN}" \
  -H "Content-Type: application/json" \
  -d "{
    \"realm\": \"${REALM}\",
    \"enabled\": true,
    \"displayName\": \"Rust Template\"
  }")

if [ $? -eq 0 ]; then
    echo "✅ Realm criado com sucesso!"
else
    echo "⚠️  Realm pode já existir ou erro na criação"
fi

# 3. Criar client
echo "🔧 Criando client '${CLIENT_ID}'..."

CLIENT_RESPONSE=$(curl -s -X POST \
  "${KEYCLOAK_URL}/admin/realms/${REALM}/clients" \
  -H "Authorization: Bearer ${ADMIN_TOKEN}" \
  -H "Content-Type: application/json" \
  -d "{
    \"clientId\": \"${CLIENT_ID}\",
    \"enabled\": true,
    \"protocol\": \"openid-connect\",
    \"publicClient\": false,
    \"standardFlowEnabled\": true,
    \"directAccessGrantsEnabled\": true,
    \"serviceAccountsEnabled\": true,
    \"redirectUris\": [\"http://localhost:3000/*\"],
    \"webOrigins\": [\"http://localhost:3000\"],
    \"clientAuthenticatorType\": \"client-secret\",
    \"secret\": \"${CLIENT_SECRET}\"
  }")

if [ $? -eq 0 ]; then
    echo "✅ Client criado com sucesso!"
else
    echo "⚠️  Client pode já existir ou erro na criação"
fi

# 4. Criar roles do realm
echo "👥 Criando roles do realm..."

ROLES=("super_admin" "admin" "user")

for role in "${ROLES[@]}"; do
    echo "   Criando role: ${role}"
    curl -s -X POST \
      "${KEYCLOAK_URL}/admin/realms/${REALM}/roles" \
      -H "Authorization: Bearer ${ADMIN_TOKEN}" \
      -H "Content-Type: application/json" \
      -d "{
        \"name\": \"${role}\",
        \"description\": \"${role} role\"
      }" > /dev/null
done

echo "✅ Roles do realm criadas!"

# 5. Criar roles do client
echo "🔐 Criando roles do client..."

# Primeiro, obter o client ID interno
CLIENT_INFO=$(curl -s -X GET \
  "${KEYCLOAK_URL}/admin/realms/${REALM}/clients?clientId=${CLIENT_ID}" \
  -H "Authorization: Bearer ${ADMIN_TOKEN}")

CLIENT_UUID=$(echo "$CLIENT_INFO" | jq -r '.[0].id')

for role in "${ROLES[@]}"; do
    echo "   Criando role do client: ${role}"
    curl -s -X POST \
      "${KEYCLOAK_URL}/admin/realms/${REALM}/clients/${CLIENT_UUID}/roles" \
      -H "Authorization: Bearer ${ADMIN_TOKEN}" \
      -H "Content-Type: application/json" \
      -d "{
        \"name\": \"${role}\",
        \"description\": \"${role} client role\"
      }" > /dev/null
done

echo "✅ Roles do client criadas!"

# 6. Criar usuário de teste
echo "👤 Criando usuário de teste..."

USER_RESPONSE=$(curl -s -X POST \
  "${KEYCLOAK_URL}/admin/realms/${REALM}/users" \
  -H "Authorization: Bearer ${ADMIN_TOKEN}" \
  -H "Content-Type: application/json" \
  -d "{
    \"username\": \"${TEST_USER_EMAIL}\",
    \"email\": \"${TEST_USER_EMAIL}\",
    \"firstName\": \"${TEST_USER_FIRST_NAME}\",
    \"lastName\": \"${TEST_USER_LAST_NAME}\",
    \"enabled\": true,
    \"emailVerified\": true,
    \"credentials\": [{
      \"type\": \"password\",
      \"value\": \"${TEST_USER_PASSWORD}\",
      \"temporary\": false
    }]
  }")

if [ $? -eq 0 ]; then
    echo "✅ Usuário de teste criado!"
    
    # Obter ID do usuário
    USER_ID=$(curl -s -X GET \
      "${KEYCLOAK_URL}/admin/realms/${REALM}/users?username=${TEST_USER_EMAIL}" \
      -H "Authorization: Bearer ${ADMIN_TOKEN}" | jq -r '.[0].id')
    
    # Adicionar role ao usuário
    echo "   Adicionando role '${TEST_USER_ROLE}' ao usuário..."
    curl -s -X POST \
      "${KEYCLOAK_URL}/admin/realms/${REALM}/users/${USER_ID}/role-mappings/realm" \
      -H "Authorization: Bearer ${ADMIN_TOKEN}" \
      -H "Content-Type: application/json" \
      -d "[{\"name\": \"${TEST_USER_ROLE}\", \"id\": \"${TEST_USER_ROLE}\"}]" > /dev/null
    
    echo "✅ Role adicionada ao usuário!"
else
    echo "⚠️  Usuário pode já existir ou erro na criação"
fi

echo ""
echo "🎉 Configuração do Keycloak concluída!"
echo ""
echo "📋 Configurações para seu .env:"
echo "KEYCLOAK_BASE_URL=http://localhost:8081"
echo "KEYCLOAK_REALM=${REALM}"
echo "KEYCLOAK_CLIENT_ID=${CLIENT_ID}"
echo "KEYCLOAK_CLIENT_SECRET=${CLIENT_SECRET}"
echo "KEYCLOAK_ADMIN_USERNAME=${ADMIN_USERNAME}"
echo "KEYCLOAK_ADMIN_PASSWORD=${ADMIN_PASSWORD}"
echo ""
echo "👤 Usuário de teste:"
echo "Email: ${TEST_USER_EMAIL}"
echo "Senha: ${TEST_USER_PASSWORD}"
echo ""
echo "🔑 Para obter token de teste, execute:"
echo "./get_keycloak_token.sh" 
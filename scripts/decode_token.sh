#!/bin/bash

# Script para decodificar JWT tokens (compatível com macOS)
# Uso: ./decode_token.sh <token>

if [ -z "$1" ]; then
    echo "❌ Uso: ./decode_token.sh <token>"
    echo "Exemplo: ./decode_token.sh eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..."
    exit 1
fi

TOKEN="$1"

echo "🔍 Decodificando JWT token..."
echo ""

# Função para decodificar base64 com padding
decode_base64() {
    local input="$1"
    
    # Remover caracteres especiais
    input=$(echo "$input" | tr -d '\n\r')
    
    # Adicionar padding se necessário
    local padding=$((4 - ${#input} % 4))
    if [ $padding -ne 4 ]; then
        input="${input}${padding}="
    fi
    
    # Tentar decodificar
    echo "$input" | base64 -d 2>/dev/null
}

# Extrair e decodificar header
echo "📋 Header:"
HEADER=$(echo "$TOKEN" | cut -d'.' -f1)
echo "Header raw: $HEADER"
HEADER_DECODED=$(echo "$HEADER" | decode_base64)
if [ $? -eq 0 ] && [ -n "$HEADER_DECODED" ]; then
    echo "$HEADER_DECODED" | jq '.' 2>/dev/null || echo "Erro ao formatar header JSON"
else
    echo "Erro ao decodificar header"
fi

echo ""
echo "📋 Payload:"
PAYLOAD=$(echo "$TOKEN" | cut -d'.' -f2)
echo "Payload raw: $PAYLOAD"
PAYLOAD_DECODED=$(echo "$PAYLOAD" | decode_base64)
if [ $? -eq 0 ] && [ -n "$PAYLOAD_DECODED" ]; then
    echo "$PAYLOAD_DECODED" | jq '.' 2>/dev/null || echo "Erro ao formatar payload JSON"
else
    echo "Erro ao decodificar payload"
fi

echo ""
echo "🔍 Informações úteis do payload:"
if [ -n "$PAYLOAD_DECODED" ]; then
    echo "   Subject (sub): $(echo "$PAYLOAD_DECODED" | jq -r '.sub // "N/A"')"
    echo "   Email: $(echo "$PAYLOAD_DECODED" | jq -r '.email // "N/A"')"
    echo "   Name: $(echo "$PAYLOAD_DECODED" | jq -r '.name // "N/A"')"
    echo "   Expires (exp): $(echo "$PAYLOAD_DECODED" | jq -r '.exp // "N/A"')"
    echo "   Issued At (iat): $(echo "$PAYLOAD_DECODED" | jq -r '.iat // "N/A"')"
    echo "   Issuer (iss): $(echo "$PAYLOAD_DECODED" | jq -r '.iss // "N/A"')"
    echo "   Audience (aud): $(echo "$PAYLOAD_DECODED" | jq -r '.aud // "N/A"')"
    
    # Mostrar roles se existirem
    REALM_ROLES=$(echo "$PAYLOAD_DECODED" | jq -r '.realm_access.roles // []')
    if [ "$REALM_ROLES" != "[]" ]; then
        echo "   Realm Roles: $REALM_ROLES"
    fi
    
    CLIENT_ROLES=$(echo "$PAYLOAD_DECODED" | jq -r '.resource_access."rust-template-client".roles // []')
    if [ "$CLIENT_ROLES" != "[]" ]; then
        echo "   Client Roles: $CLIENT_ROLES"
    fi
else
    echo "   Erro ao decodificar payload"
fi 
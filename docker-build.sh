#!/bin/bash

# Script para build e deploy da aplicação Docker
# Uso: ./docker-build.sh [comando]

set -e

IMAGE_NAME="rust-template"
TAG=${TAG:-"latest"}

case "${1:-help}" in
    "build")
        echo "🔨 Construindo imagem Docker..."
        docker build -t ${IMAGE_NAME}:${TAG} .
        echo "✅ Imagem construída: ${IMAGE_NAME}:${TAG}"
        ;;
    
    "run")
        echo "🚀 Executando aplicação..."
        docker-compose up -d
        echo "✅ Aplicação iniciada!"
        echo "📋 URL: http://localhost:8080"
        echo ""
        echo "⚠️  Lembre-se de configurar as variáveis de ambiente:"
        echo "  - DATABASE_URL"
        echo "  - JWT_SECRET"
        echo "  - ELASTICSEARCH_URL"
        echo "  - KEYCLOAK_BASE_URL"
        echo "  - ETC..."
        ;;
    
    "stop")
        echo "🛑 Parando aplicação..."
        docker-compose down
        echo "✅ Aplicação parada!"
        ;;
    
    "logs")
        echo "📋 Exibindo logs..."
        docker-compose logs -f
        ;;
    
    "clean")
        echo "🧹 Limpando containers..."
        docker-compose down
        docker system prune -f
        echo "✅ Limpeza concluída!"
        ;;
    
    "dev")
        echo "🔧 Modo desenvolvimento..."
        docker-compose up --build
        ;;
    
    "prod")
        echo "🚀 Modo produção..."
        # Build da imagem
        docker build -t ${IMAGE_NAME}:${TAG} .
        
        # Executar apenas a aplicação
        docker run -d \
            --name rust-template-prod \
            -p 8080:8080 \
            -e APP_ENVIRONMENT=production \
            -e SERVER_HOST=0.0.0.0 \
            -e SERVER_PORT=8080 \
            -e DATABASE_URL=${DATABASE_URL} \
            -e JWT_SECRET=${JWT_SECRET} \
            -e JWT_EXPIRES_IN=${JWT_EXPIRES_IN:-86400} \
            -e DATABASE_MAX_CONNECTIONS=${DATABASE_MAX_CONNECTIONS:-10} \
            -e ELASTICSEARCH_URL=${ELASTICSEARCH_URL} \
            -e ELASTICSEARCH_INDEX_PREFIX=${ELASTICSEARCH_INDEX_PREFIX} \
            -e KEYCLOAK_BASE_URL=${KEYCLOAK_BASE_URL} \
            -e KEYCLOAK_REALM=${KEYCLOAK_REALM} \
            -e KEYCLOAK_CLIENT_ID=${KEYCLOAK_CLIENT_ID} \
            -e KEYCLOAK_CLIENT_SECRET=${KEYCLOAK_CLIENT_SECRET} \
            -e KEYCLOAK_ADMIN_USERNAME=${KEYCLOAK_ADMIN_USERNAME} \
            -e KEYCLOAK_ADMIN_PASSWORD=${KEYCLOAK_ADMIN_PASSWORD} \
            ${IMAGE_NAME}:${TAG}
        
        echo "✅ Aplicação em produção iniciada!"
        echo "📋 URL: http://localhost:8080"
        ;;
    
    "help"|*)
        echo "📋 Uso: ./docker-build.sh [comando]"
        echo ""
        echo "Comandos disponíveis:"
        echo "  build   - Construir imagem Docker"
        echo "  run     - Executar com Docker Compose"
        echo "  stop    - Parar aplicação"
        echo "  logs    - Exibir logs"
        echo "  clean   - Limpar containers"
        echo "  dev     - Modo desenvolvimento (logs em tempo real)"
        echo "  prod    - Modo produção (apenas aplicação)"
        echo "  help    - Exibir esta ajuda"
        echo ""
        echo "Exemplos:"
        echo "  ./docker-build.sh build"
        echo "  ./docker-build.sh run"
        echo "  TAG=v1.0.0 ./docker-build.sh build"
        echo ""
        echo "⚠️  Variáveis de ambiente necessárias:"
        echo "  - DATABASE_URL"
        echo "  - JWT_SECRET"
        echo "  - ELASTICSEARCH_URL"
        echo "  - KEYCLOAK_BASE_URL"
        echo "  - ETC..."
        ;;
esac 
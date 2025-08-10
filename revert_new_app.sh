#!/bin/bash

# Verificar se o nome do app foi fornecido
if [ $# -eq 0 ]; then
    echo "❌ Erro: Nome do app é obrigatório"
    echo "Uso: ./revert_new_app.sh NOME_APP"
    echo ""
    echo "Exemplo: ./revert_new_app.sh User"
    exit 1
fi

APP_NAME=$1

# Converter para lowercase para o nome do diretório
APP_DIR_NAME=$(echo "$APP_NAME" | tr '[:upper:]' '[:lower:]')
# Converter para plural para a tabela
TABLE_NAME="${APP_DIR_NAME}s"

echo "🔄 Revertendo app: $APP_NAME"
echo "📁 Diretório: src/apps/$APP_DIR_NAME"
echo "🗄️  Tabela: $TABLE_NAME"
echo ""

# Confirmar a ação
read -p "⚠️  Tem certeza que deseja remover o app '$APP_NAME'? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Operação cancelada"
    exit 1
fi

# 1. Remover diretório do app
if [ -d "src/apps/$APP_DIR_NAME" ]; then
    rm -rf "src/apps/$APP_DIR_NAME"
    echo "✅ Diretório removido: src/apps/$APP_DIR_NAME"
else
    echo "⚠️  Diretório não encontrado: src/apps/$APP_DIR_NAME"
fi

# 2. Remover migration relacionada
if [ -d "migrations" ]; then
    # Encontrar arquivos de migration que contenham o nome da tabela
    MIGRATION_FILES=$(find migrations -name "*create_${TABLE_NAME}.sql" 2>/dev/null)
    
    if [ -n "$MIGRATION_FILES" ]; then
        for file in $MIGRATION_FILES; do
            rm "$file"
            echo "✅ Migration removida: $file"
        done
    else
        echo "⚠️  Nenhuma migration encontrada para: create_${TABLE_NAME}"
    fi
else
    echo "⚠️  Diretório migrations não encontrado"
fi

# 3. Remover declaração do módulo do src/apps/mod.rs
if [ -f "src/apps/mod.rs" ]; then
    # Remover a linha que declara o módulo
    sed -i.bak "/pub mod ${APP_DIR_NAME};/d" "src/apps/mod.rs"
    
    # Remover arquivo de backup do sed
    rm -f "src/apps/mod.rs.bak"
    
    echo "✅ Declaração do módulo removida de src/apps/mod.rs"
    
    # Verificar se o arquivo ficou vazio e removê-lo se necessário
    if [ ! -s "src/apps/mod.rs" ]; then
        rm "src/apps/mod.rs"
        echo "✅ Arquivo src/apps/mod.rs removido (ficou vazio)"
    fi
else
    echo "⚠️  Arquivo src/apps/mod.rs não encontrado"
fi

# 4. Verificar se o diretório src/apps ficou vazio
if [ -d "src/apps" ] && [ -z "$(ls -A src/apps)" ]; then
    rm -rf "src/apps"
    echo "✅ Diretório src/apps removido (ficou vazio)"
fi

echo ""
echo "🎉 App '$APP_NAME' removido com sucesso!"
echo ""
echo "📋 Resumo das ações:"
echo "   ✅ Diretório src/apps/${APP_DIR_NAME}/ removido"
echo "   ✅ Migrations relacionadas removidas"
echo "   ✅ Declaração do módulo removida"
echo ""
echo "💡 Dica: Se você executou as migrations no banco de dados,"
echo "   você precisará reverter manualmente usando:"
echo "   sqlx migrate revert"
echo "" 
#!/bin/bash

# Verificar se o nome do app foi fornecido
if [ $# -eq 0 ]; then
    echo "❌ Erro: Nome do app é obrigatório"
    echo "Uso: ./start_new_app.sh NOME_NOVO_APP [--no_migrate]"
    exit 1
fi

APP_NAME=$1
NO_MIGRATE=false

# Verificar se o parâmetro --no_migrate foi passado
if [ "$2" = "--no_migrate" ]; then
    NO_MIGRATE=true
fi

# Converter para lowercase para o nome do diretório
APP_DIR_NAME=$(echo "$APP_NAME" | tr '[:upper:]' '[:lower:]')
# Converter para plural para a tabela
TABLE_NAME="${APP_DIR_NAME}s"

echo "🚀 Criando novo app: $APP_NAME"
echo "📁 Diretório: src/apps/$APP_DIR_NAME"
echo "🗄️  Tabela: $TABLE_NAME"

# 1. Criar diretório do app
mkdir -p "src/apps/$APP_DIR_NAME"
echo "✅ Diretório criado: src/apps/$APP_DIR_NAME"

# 2. Criar routes.rs
cat > "src/apps/$APP_DIR_NAME/routes.rs" << EOF
use actix_web::{web, HttpResponse, Responder};
use crate::app_core::app_extensions::RequestUserExt;
use crate::app_core::app_state::AppState;
use crate::app_core::app_error::AppError;
use uuid::Uuid;

pub async fn list_${APP_DIR_NAME}s(
    app_state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let user_id = req.user_id()?;
    let tenant_id = req.tenant_id()?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Lista de $APP_NAME",
        "data": []
    })))
}

pub async fn get_${APP_DIR_NAME}(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let user_id = req.user_id()?;
    let tenant_id = req.tenant_id()?;
    let id = path.into_inner();
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Detalhes do $APP_NAME",
        "id": id,
        "data": {}
    })))
}

pub async fn create_${APP_DIR_NAME}(
    app_state: web::Data<AppState>,
    payload: web::Json<serde_json::Value>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let user_id = req.user_id()?;
    let tenant_id = req.tenant_id()?;
    Ok(HttpResponse::Created().json(serde_json::json!({
        "message": "$APP_NAME criado com sucesso",
        "data": payload.into_inner()
    })))
}

pub async fn update_${APP_DIR_NAME}(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
    payload: web::Json<serde_json::Value>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let user_id = req.user_id()?;
    let tenant_id = req.tenant_id()?;
    let id = path.into_inner();
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "$APP_NAME atualizado com sucesso",
        "id": id,
        "data": payload.into_inner()
    })))
}

pub async fn delete_${APP_DIR_NAME}(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let user_id = req.user_id()?;
    let tenant_id = req.tenant_id()?;
    let id = path.into_inner();
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "$APP_NAME deletado com sucesso",
        "id": id
    })))
}
EOF
echo "✅ routes.rs criado"

# 3. Criar models.rs
cat > "src/apps/$APP_DIR_NAME/models.rs" << EOF
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ${APP_NAME} {
    pub id: Uuid,
    pub dt_created: DateTime<Utc>,
    pub dt_updated: DateTime<Utc>,
    pub dt_deleted: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct Create${APP_NAME}Request {
    // Adicione os campos necessários aqui
}

#[derive(Debug, Deserialize)]
pub struct Update${APP_NAME}Request {
    // Adicione os campos necessários aqui
}

#[derive(Debug, Serialize)]
pub struct ${APP_NAME}Response {
    pub id: Uuid,
    pub dt_created: DateTime<Utc>,
    pub dt_updated: DateTime<Utc>,
}
EOF
echo "✅ models.rs criado"

# 4. Criar services.rs
cat > "src/apps/$APP_DIR_NAME/services.rs" << EOF
use crate::app_core::app_state::AppState;
use crate::app_core::app_error::AppError;
use crate::utils::pagination::PaginatedResponse;
use crate::apps::${APP_DIR_NAME}::models::{${APP_NAME}, Create${APP_NAME}Request, Update${APP_NAME}Request};
use crate::apps::${APP_DIR_NAME}::repositories::${APP_NAME}Repository;
use uuid::Uuid;

pub struct ${APP_NAME}Service;

impl ${APP_NAME}Service {
    pub async fn list_${APP_DIR_NAME}s(app_state: &AppState) -> Result<PaginatedResponse<${APP_NAME}>, AppError> {
        let repository = ${APP_NAME}Repository::new(app_state);
        let ${APP_DIR_NAME}s = repository.find_all().await
            .map_err(|e| AppError::database_error(e.to_string()))?;
        
        Ok(PaginatedResponse {
            count: ${APP_DIR_NAME}s.len() as i64,
            results: ${APP_DIR_NAME}s,
            limit: 10,
            offset: 0,
        })
    }

    pub async fn get_${APP_DIR_NAME}(app_state: &AppState, id: Uuid) -> Result<Option<${APP_NAME}>, AppError> {
        let repository = ${APP_NAME}Repository::new(app_state);
        repository.find_by_id(id).await
            .map_err(|e| AppError::database_error(e.to_string()))
    }

    pub async fn create_${APP_DIR_NAME}(app_state: &AppState, request: Create${APP_NAME}Request) -> Result<${APP_NAME}, AppError> {
        let repository = ${APP_NAME}Repository::new(app_state);
        repository.create(request).await
            .map_err(|e| AppError::database_error(e.to_string()))
    }

    pub async fn update_${APP_DIR_NAME}(app_state: &AppState, id: Uuid, request: Update${APP_NAME}Request) -> Result<Option<${APP_NAME}>, AppError> {
        let repository = ${APP_NAME}Repository::new(app_state);
        repository.update(id, request).await
            .map_err(|e| AppError::database_error(e.to_string()))
    }

    pub async fn delete_${APP_DIR_NAME}(app_state: &AppState, id: Uuid) -> Result<bool, AppError> {
        let repository = ${APP_NAME}Repository::new(app_state);
        repository.delete(id).await
            .map_err(|e| AppError::database_error(e.to_string()))
    }
}
EOF
echo "✅ services.rs criado"

# 5. Criar repositories.rs
cat > "src/apps/$APP_DIR_NAME/repositories.rs" << EOF
use crate::app_core::app_state::AppState;
use crate::apps::${APP_DIR_NAME}::models::{${APP_NAME}, Create${APP_NAME}Request, Update${APP_NAME}Request};
use sqlx::Row;
use uuid::Uuid;
use chrono::{Utc, DateTime, NaiveDateTime};

pub struct ${APP_NAME}Repository<'a> {
    app_state: &'a AppState,
}

impl<'a> ${APP_NAME}Repository<'a> {
    pub fn new(app_state: &'a AppState) -> Self {
        Self { app_state }
    }

    pub async fn find_all(&self) -> Result<Vec<${APP_NAME}>, sqlx::Error> {
        let rows = sqlx::query!(
            "SELECT id, dt_created, dt_updated, dt_deleted FROM ${TABLE_NAME} WHERE dt_deleted IS NULL"
        )
        .fetch_all(&self.app_state.db)
        .await?;

        let ${APP_DIR_NAME}s = rows
            .into_iter()
            .map(|row| ${APP_NAME} {
                id: row.id,
                dt_created: DateTime::from_naive_utc_and_offset(row.dt_created, Utc),
                dt_updated: DateTime::from_naive_utc_and_offset(row.dt_updated, Utc),
                dt_deleted: row.dt_deleted.map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
            })
            .collect();

        Ok(${APP_DIR_NAME}s)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<${APP_NAME}>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT id, dt_created, dt_updated, dt_deleted FROM ${TABLE_NAME} WHERE id = \$1 AND dt_deleted IS NULL",
            id
        )
        .fetch_optional(&self.app_state.db)
        .await?;

        Ok(row.map(|row| ${APP_NAME} {
            id: row.id,
            dt_created: DateTime::from_naive_utc_and_offset(row.dt_created, Utc),
            dt_updated: DateTime::from_naive_utc_and_offset(row.dt_updated, Utc),
            dt_deleted: row.dt_deleted.map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
        }))
    }

    pub async fn create(&self, _request: Create${APP_NAME}Request) -> Result<${APP_NAME}, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let row = sqlx::query!(
            "INSERT INTO ${TABLE_NAME} (id, dt_created, dt_updated) VALUES (\$1, \$2, \$3) RETURNING id, dt_created, dt_updated, dt_deleted",
            id,
            now.naive_utc(),
            now.naive_utc()
        )
        .fetch_one(&self.app_state.db)
        .await?;

        Ok(${APP_NAME} {
            id: row.id,
            dt_created: DateTime::from_naive_utc_and_offset(row.dt_created, Utc),
            dt_updated: DateTime::from_naive_utc_and_offset(row.dt_updated, Utc),
            dt_deleted: row.dt_deleted.map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
        })
    }

    pub async fn update(&self, id: Uuid, _request: Update${APP_NAME}Request) -> Result<Option<${APP_NAME}>, sqlx::Error> {
        let now = Utc::now();

        let row = sqlx::query!(
            "UPDATE ${TABLE_NAME} SET dt_updated = \$1 WHERE id = \$2 AND dt_deleted IS NULL RETURNING id, dt_created, dt_updated, dt_deleted",
            now.naive_utc(),
            id
        )
        .fetch_optional(&self.app_state.db)
        .await?;

        Ok(row.map(|row| ${APP_NAME} {
            id: row.id,
            dt_created: DateTime::from_naive_utc_and_offset(row.dt_created, Utc),
            dt_updated: DateTime::from_naive_utc_and_offset(row.dt_updated, Utc),
            dt_deleted: row.dt_deleted.map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
        }))
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let now = Utc::now();

        let result = sqlx::query!(
            "UPDATE ${TABLE_NAME} SET dt_deleted = \$1 WHERE id = \$2 AND dt_deleted IS NULL",
            now.naive_utc(),
            id
        )
        .execute(&self.app_state.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
EOF
echo "✅ repositories.rs criado"

# 6. Criar tests.rs
cat > "src/apps/$APP_DIR_NAME/tests.rs" << EOF
#[cfg(test)]
mod tests {
    use super::*;
    use crate::apps::${APP_DIR_NAME}::models::${APP_NAME};

    #[test]
    fn test_${APP_DIR_NAME}_creation() {
        // Adicione seus testes aqui
        assert!(true);
    }

    #[test]
    fn test_${APP_DIR_NAME}_validation() {
        // Adicione seus testes aqui
        assert!(true);
    }
}
EOF
echo "✅ tests.rs criado"

# 7. Criar mod.rs
cat > "src/apps/$APP_DIR_NAME/mod.rs" << EOF
pub mod models;
pub mod routes;
pub mod services;
pub mod repositories;

#[cfg(test)]
mod tests;
EOF
echo "✅ mod.rs criado"

# 8. Criar migration se --no_migrate não foi passado
if [ "$NO_MIGRATE" = false ]; then
    # Criar diretório migrations se não existir
    mkdir -p migrations
    
    # Gerar timestamp para o nome do arquivo
    TIMESTAMP=$(date +%Y%m%d%H%M%S)
    MIGRATION_FILE="migrations/${TIMESTAMP}_create_${TABLE_NAME}.sql"
    
    cat > "$MIGRATION_FILE" << EOF
-- Migration: create_${TABLE_NAME}
-- Created at: $(date)

CREATE TABLE ${TABLE_NAME} (
    id UUID PRIMARY KEY,
    dt_created TIMESTAMP NOT NULL DEFAULT now(),
    dt_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    dt_deleted TIMESTAMP
);

-- Índices para melhor performance
CREATE INDEX idx_${TABLE_NAME}_dt_created ON ${TABLE_NAME}(dt_created);
CREATE INDEX idx_${TABLE_NAME}_dt_deleted ON ${TABLE_NAME}(dt_deleted);
EOF
    echo "✅ Migration criada: $MIGRATION_FILE"
else
    echo "⏭️  Pulando criação da migration (--no_migrate)"
fi

# 9. Atualizar src/apps/mod.rs
if [ ! -f "src/apps/mod.rs" ]; then
    echo "pub mod ${APP_DIR_NAME};" > "src/apps/mod.rs"
else
    # Verificar se o módulo já existe
    if ! grep -q "pub mod ${APP_DIR_NAME};" "src/apps/mod.rs"; then
        echo "pub mod ${APP_DIR_NAME};" >> "src/apps/mod.rs"
    fi
fi
echo "✅ src/apps/mod.rs atualizado"

echo ""
echo "🎉 App '$APP_NAME' criado com sucesso!"
echo ""
echo "📁 Estrutura criada:"
echo "   src/apps/${APP_DIR_NAME}/"
echo "   ├── mod.rs"
echo "   ├── models.rs"
echo "   ├── routes.rs"
echo "   ├── services.rs"
echo "   ├── repositories.rs"
echo "   └── tests.rs"
echo ""
if [ "$NO_MIGRATE" = false ]; then
    echo "🗄️  Migration criada: $MIGRATION_FILE"
fi
echo ""
echo "🔧 Próximos passos:"
echo "   1. Adicione os campos necessários em models.rs"
echo "   2. Implemente a lógica de negócio em services.rs"
echo "   3. Configure as rotas em src/app_core/routes.rs"
echo "   4. Execute a migration: sqlx migrate run"
echo "" 
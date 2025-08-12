# Rust E-commerce - Actix Web

Uma aplicação de estudo de e-commerce desenvolvida em Rust com Actix Web, PostgreSQL, MongoDB e Elasticsearch. Este projeto demonstra a implementação de uma arquitetura robusta para sistemas de comércio eletrônico, incluindo gestão de produtos, usuários, tenants e autenticação.

## Configuração

### 1. Variáveis de Ambiente

Crie um arquivo `.env` na raiz do projeto com as seguintes variáveis:

```env
# Configurações do Servidor
SERVER_HOST=127.0.0.1
SERVER_PORT=8080

# Ambiente da aplicação
APP_ENVIRONMENT=development

# Configurações do Banco de Dados PostgreSQL
DATABASE_URL=postgresql://username:password@localhost:5432/database_name
DATABASE_MAX_CONNECTIONS=5

# Configurações do MongoDB
MONGO_URI=mongodb://localhost:27017

# Configurações do Elasticsearch
ELASTICSEARCH_URL=http://localhost:9200
ELASTICSEARCH_INDEX_PREFIX=app

# Configurações JWT
JWT_SECRET=your_super_secret_jwt_key_that_is_at_least_32_characters_long
JWT_EXPIRES_IN=86400
```

### 2. Dependências Externas

Certifique-se de ter os seguintes serviços rodando:

- **PostgreSQL**: Banco de dados principal
- **MongoDB**: Banco de dados NoSQL
- **Elasticsearch**: Motor de busca

### 3. Executando o Projeto

```bash
# Verificar se compila
cargo check

# Executar em modo desenvolvimento
cargo run

# Executar em modo release
cargo run --release
```

O servidor estará disponível em `http://127.0.0.1:8080`

## Scripts de Desenvolvimento

### Criando um Novo App

Use o script `start_new_app.sh` para criar rapidamente um novo módulo de aplicação:

```bash
# Criar um app com migration
./start_new_app.sh User

# Criar um app sem migration
./start_new_app.sh Product --no_migrate
```

O script criará automaticamente:
- 📁 `src/apps/user/` (diretório do app)
- 📄 `mod.rs` (declarações de módulos)
- 📄 `models.rs` (modelos de dados)
- 📄 `routes.rs` (rotas da API)
- 📄 `services.rs` (lógica de negócio)
- 📄 `repositories.rs` (acesso ao banco)
- 📄 `tests.rs` (testes unitários)
- 🗄️ `migrations/YYYYMMDDHHMMSS_create_users.sql` (migration do banco)

### Revertendo um App

Se precisar remover um app criado, use o script `revert_new_app.sh`:

```bash
./revert_new_app.sh User
```

O script irá:
- ❌ Remover o diretório do app
- ❌ Remover as migrations relacionadas
- ❌ Remover a declaração do módulo
- ⚠️ Pedir confirmação antes de executar

## 🚀 Melhorias Implementadas no Script

### Arquitetura de Tratamento de Erros

O script agora gera código com uma arquitetura robusta de tratamento de erros em três camadas:

#### 1. **Routes Layer** - `Result<impl Responder, AppError>`

**Antes:**
```rust
pub async fn list_items(app_state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Lista de Item",
        "data": []
    }))
}
```

**Depois:**
```rust
pub async fn list_items(app_state: web::Data<AppState>) -> Result<impl Responder, AppError> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Lista de Item",
        "data": []
    })))
}
```

**Benefícios:**
- ✅ **Tratamento de erros automático**: Actix Web converte `AppError` em respostas HTTP apropriadas
- ✅ **Consistência**: Todas as rotas seguem o mesmo padrão de retorno
- ✅ **Debugging melhorado**: Erros são rastreados até a origem
- ✅ **Respostas padronizadas**: Erros seguem o formato definido em `AppError`

#### 2. **Services Layer** - `Result<PaginatedResponse<T>, AppError>`

**Antes:**
```rust
pub async fn list_users(app_state: &AppState) -> Result<Vec<User>, Box<dyn std::error::Error>> {
    let repository = UserRepository::new(app_state);
    repository.find_all().await
}
```

**Depois:**
```rust
pub async fn list_items(app_state: &AppState) -> Result<PaginatedResponse<Item>, AppError> {
    let repository = ItemRepository::new(app_state);
    let items = repository.find_all().await
        .map_err(|e| AppError::database_error(e.to_string()))?;
    
    Ok(PaginatedResponse {
        count: items.len() as i64,
        results: items,
        limit: 10,
        offset: 0,
    })
}
```

**Benefícios:**
- ✅ **Paginação nativa**: Todas as listagens retornam dados paginados
- ✅ **Metadados úteis**: Inclui `count`, `limit`, `offset` para frontend
- ✅ **Padrão consistente**: Formato uniforme para todas as APIs de listagem
- ✅ **Melhor UX**: Frontend pode implementar paginação facilmente
- ✅ **Conversão de erros**: Erros de banco são convertidos para `AppError`

#### 3. **Repositories Layer** - `Result<T, sqlx::Error>`

**Antes:**
```rust
pub async fn find_all(&self) -> Result<Vec<User>, Box<dyn std::error::Error>> {
    // código...
}
```

**Depois:**
```rust
pub async fn find_all(&self) -> Result<Vec<User>, sqlx::Error> {
    // código...
}
```

**Benefícios:**
- ✅ **Tipagem específica**: Erro específico do SQLx em vez de erro genérico
- ✅ **Melhor performance**: Evita boxing de erros desnecessário
- ✅ **Debugging preciso**: Erros de banco são mais específicos e informativos
- ✅ **Consistência com SQLx**: Usa o tipo de erro nativo da biblioteca

### Fluxo de Erros Atualizado

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Repository    │    │     Service     │    │      Route      │    │   HTTP Client   │
│                 │    │                 │    │                 │    │                 │
│ sqlx::Error     │───▶│ AppError        │───▶│ Result<impl     │───▶│ HTTP Response   │
│                 │    │                 │    │  Responder,     │    │ (com status     │
│                 │    │                 │    │  AppError>      │    │  apropriado)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘    └─────────────────┘
        │                       │                       │                       │
        │ map_err()             │ automatic conversion  │ actix-web handles     │
        ▼                       ▼                       ▼                       ▼
   Database errors      Business logic errors    Route errors         Client response
```

### Estruturas de Dados Utilizadas

#### `AppError` - Tratamento Centralizado de Erros
```rust
pub enum AppError {
    Conflict(Option<String>),
    DatabaseError(Option<String>),
    NotFound(Option<String>),
    Unauthorized(Option<String>),
    BadRequest(Option<String>),
    InternalError(Option<String>),
}
```

#### `PaginatedResponse<T>` - Resposta Paginada Padrão
```rust
pub struct PaginatedResponse<T> {
    pub count: i64,      // Total de registros
    pub results: Vec<T>, // Dados da página atual
    pub limit: i64,      // Limite por página
    pub offset: i64,     // Offset da página
}
```

### Benefícios Gerais da Nova Arquitetura

1. **🔒 Tratamento de Erros Robusto**
   - Cada camada tem responsabilidade específica
   - Erros são propagados de forma controlada
   - Respostas HTTP consistentes

2. **📊 Paginação Padrão**
   - Todas as listagens são paginadas automaticamente
   - Metadados úteis para frontend
   - Performance otimizada para grandes datasets

3. **⚡ Performance Melhorada**
   - Menos uso de `Box<dyn Error>` genérico
   - Tipagem específica para erros de banco
   - Conversões eficientes entre tipos

4. **🛠️ Manutenibilidade**
   - Código mais previsível e fácil de manter
   - Padrões consistentes em toda a aplicação
   - Debugging simplificado

5. **🎯 Experiência do Desenvolvedor**
   - Script gera código pronto para produção
   - Menos boilerplate para implementar
   - Estrutura escalável e profissional

### Exemplo de Uso Completo

```bash
# Criar um novo app com a nova arquitetura
./start_new_app.sh Product

# O script gera automaticamente:
# - Routes com Result<impl Responder, AppError>
# - Services com Result<PaginatedResponse<T>, AppError>
# - Repositories com Result<T, sqlx::Error>
# - Tratamento de erros em todas as camadas
# - Paginação automática para listagens
```

## Estrutura do Projeto

```
src/
├── app_core/           # Núcleo da aplicação
│   ├── app_state.rs    # Estado global da aplicação
│   ├── app_error.rs    # Tratamento centralizado de erros
│   ├── databases/      # Configurações de banco de dados
│   ├── app_routes.rs   # Definição de rotas
│   ├── settings.rs     # Configurações da aplicação
│   └── ...
├── apps/               # Módulos da aplicação
│   ├── user/           # Sistema de usuários e autenticação
│   │   ├── mod.rs
│   │   ├── models.rs
│   │   ├── routes.rs   # Result<impl Responder, AppError>
│   │   ├── services.rs # Result<PaginatedResponse<T>, AppError>
│   │   ├── repositories.rs # Result<T, sqlx::Error>
│   │   ├── keycloak/   # Integração com Keycloak
│   │   └── tests.rs
│   ├── product/        # 🛍️ Sistema de produtos (E-commerce)
│   │   ├── mod.rs
│   │   ├── models.rs   # Product, CreateProductRequest, UpdateProductRequest
│   │   ├── routes.rs   # CRUD completo com tratamento de erros
│   │   ├── services.rs # Lógica de negócio e validações
│   │   ├── repositories.rs # Acesso ao banco PostgreSQL
│   │   └── tests.rs    # Testes abrangentes
│   ├── tenant/         # Sistema de multi-tenancy
│   ├── orchestrator/   # Gestão de processos de negócio
│   ├── sync_app/       # Sistema de sincronização
│   └── mod.rs
├── utils/              # Utilitários
│   ├── pagination.rs   # PaginatedResponse<T>
│   ├── validation.rs   # Validadores customizados
│   ├── jwt.rs          # Gestão de tokens JWT
│   └── ...
└── main.rs            # Ponto de entrada
```

## Funcionalidades

### 🏗️ **Arquitetura e Infraestrutura**
- ✅ Servidor Actix Web configurado
- ✅ Conexão com PostgreSQL (banco principal)
- ✅ Conexão com MongoDB (dados NoSQL)
- ✅ Conexão com Elasticsearch (busca e indexação)
- ✅ Sistema de configurações por ambiente
- ✅ Logging com tracing
- ✅ Estrutura modular escalável
- ✅ Scripts de automação para criação de apps
- ✅ Sistema de migrations automático

### 🔧 **Qualidade de Código**
- ✅ **Tratamento robusto de erros com AppError**
- ✅ **Paginação automática com PaginatedResponse**
- ✅ **Tipagem específica para erros de banco**
- ✅ **Arquitetura em camadas com responsabilidades bem definidas**
- ✅ **Sistema de gestão de tokens para autenticação e recuperação**
- ✅ **Validadores customizados para email, senha, telefone, CPF e data**

### 🛍️ **Módulos de E-commerce**
- ✅ **Sistema de Usuários**: Cadastro, login, perfis e autenticação
- ✅ **Sistema de Produtos**: CRUD completo com gestão de estoque e preços
- ✅ **Sistema de Tenants**: Multi-tenancy para diferentes lojas
- ✅ **Sistema de Orquestradores**: Gestão de processos de negócio
- ✅ **Sistema de Sincronização**: Produtor/consumidor para eventos

## 🔍 Sistema de Validação Customizada

O projeto inclui validadores customizados para garantir a qualidade dos dados de entrada.

### Validadores Disponíveis

| Validador | Função | Regras |
|-----------|--------|--------|
| **Email** | `validate_email` | Regex: `^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$` |
| **Password** | `validate_password` | Mínimo 8 caracteres, letras e números obrigatórios |
| **Phone** | `validate_phone` | Formato internacional: `+5511999999999` |
| **Document** | `validate_document` | CPF: `000.000.000-00` |
| **Birth Date** | `validate_birth_date` | Data: `YYYY-MM-DD` |

### Uso nos Modelos

```rust
use crate::utils::validation::{validate_email, validate_password, validate_phone, validate_document, validate_birth_date};

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UserRequest {
    #[validate(custom = "validate_email")]
    pub email: String,

    #[validate(custom = "validate_password")]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate, Serialize)]
pub struct ProfileRequest {
    #[validate(custom = "validate_phone")]
    pub phone: Option<String>,

    #[validate(custom = "validate_document")]
    pub document: Option<String>,

    #[validate(custom = "validate_birth_date")]
    pub birth_date: Option<String>,
}
```

### Regras de Validação Detalhadas

#### **Email** (`validate_email`)
- Formato padrão de email
- Aceita caracteres especiais: `.`, `_`, `%`, `+`, `-`
- Domínio válido com TLD de pelo menos 2 caracteres

#### **Password** (`validate_password`)
- **Mínimo 8 caracteres**
- **Pelo menos uma letra** (maiúscula ou minúscula)
- **Pelo menos um número**
- Aceita caracteres especiais: `@`, `$`, `!`, `%`, `*`, `#`, `?`, `&`

#### **Phone** (`validate_phone`)
- Formato internacional
- Opcional: `+` no início
- 1-15 dígitos numéricos
- Exemplo: `+5511999999999`

#### **Document** (`validate_document`)
- Formato CPF brasileiro
- Padrão: `000.000.000-00`
- Pontos e hífen obrigatórios

#### **Birth Date** (`validate_birth_date`)
- Formato ISO: `YYYY-MM-DD`
- Data válida (não aceita datas inexistentes)
- Exemplo: `1990-12-25`

### Benefícios

- ✅ **Validação consistente** em toda a aplicação
- ✅ **Mensagens de erro personalizadas** em português
- ✅ **Regras específicas** para o contexto brasileiro
- ✅ **Reutilização** em múltiplos modelos
- ✅ **Manutenibilidade** centralizada

## 🛍️ **Módulo de Produtos (E-commerce)**

O módulo de produtos é o coração do sistema de e-commerce, implementando todas as funcionalidades necessárias para gestão de catálogo de produtos.

### **Funcionalidades do Módulo**

| Operação | Endpoint | Método | Descrição |
|----------|----------|---------|-----------|
| **Listar Produtos** | `/api/v1/products/` | `GET` | Lista paginada com filtros (nome, preço, estoque) |
| **Buscar Produto** | `/api/v1/products/{id}` | `GET` | Busca produto específico por ID |
| **Criar Produto** | `/api/v1/products/` | `POST` | Cria novo produto com validações |
| **Atualizar Produto** | `/api/v1/products/{id}` | `PUT` | Atualiza produto existente |
| **Deletar Produto** | `/api/v1/products/{id}` | `DELETE` | Remove produto (soft delete) |

### **Modelo de Dados**

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Product {
    pub id: Uuid,                    // Identificador único
    pub tenant_id: Uuid,             // ID do tenant/loja
    pub name: String,                // Nome do produto
    pub slug: String,                // URL amigável
    pub short_description: Option<String>, // Descrição curta
    pub description: Option<String>,       // Descrição completa
    pub price: BigDecimal,           // Preço em centavos
    pub stock_quantity: i32,        // Quantidade em estoque
    pub attributes: Option<serde_json::Value>, // Atributos customizados
    pub is_active: bool,             // Status ativo/inativo
    pub dt_created: DateTime<Utc>,   // Data de criação
    pub dt_updated: DateTime<Utc>,   // Data de atualização
    pub dt_deleted: Option<DateTime<Utc>>, // Soft delete
}
```

### **Validações Implementadas**

- ✅ **Nome**: Obrigatório, não pode ser vazio
- ✅ **Preço**: Deve ser positivo (em centavos)
- ✅ **Estoque**: Deve ser não-negativo
- ✅ **Tenant**: Produtos são isolados por loja
- ✅ **Slug**: Geração automática baseada no nome

### **Tratamento de Erros**

O módulo implementa tratamento robusto de erros:

```rust
// Service retorna erro 404 quando produto não é encontrado
pub async fn get_product(app_state: &AppState, id: Uuid) -> Result<Product, AppError> {
    let repository = ProductRepository::new(app_state);
    let product = repository.find_by_id(id).await?;

    match product {
        Some(product) => Ok(product),
        None => Err(AppError::not_found("Produto não encontrado")),
    }
}
```

### **Respostas HTTP**

| Status | Operação | Body |
|--------|----------|------|
| **200** | Listar/Buscar/Atualizar | JSON com dados do produto |
| **201** | Criar | JSON com mensagem de sucesso e dados |
| **204** | Deletar | Sem conteúdo (sucesso) |
| **404** | Produto não encontrado | JSON com mensagem de erro |
| **400** | Dados inválidos | JSON com detalhes da validação |
| **500** | Erro interno | JSON com mensagem de erro |

### **Filtros de Listagem**

```rust
#[derive(Debug, Deserialize)]
pub struct ProductListParams {
    pub name: Option<String>,        // Filtrar por nome
    pub min_price: Option<i64>,      // Preço mínimo
    pub max_price: Option<i64>,      // Preço máximo
    pub limit: Option<i64>,          // Limite por página
    pub offset: Option<i64>,         // Offset para paginação
    pub is_active: Option<bool>,     // Filtrar por status
}
```

### **Exemplo de Uso**

```bash
# Listar produtos ativos com preço entre R$ 10 e R$ 100
GET /api/v1/products/?min_price=1000&max_price=10000&is_active=true&limit=20

# Buscar produto específico
GET /api/v1/products/550e8400-e29b-41d4-a716-446655440000

# Criar novo produto
POST /api/v1/products/
{
    "name": "Smartphone XYZ",
    "short_description": "Smartphone de última geração",
    "description": "Smartphone com câmera de 48MP...",
    "price": 199900,  // R$ 1.999,00
    "stock_quantity": 50,
    "is_active": true
}
```

### **Testes Implementados**

O módulo inclui testes abrangentes:

- ✅ **Testes de Models**: Validação de criação e atualização
- ✅ **Testes de Services**: Lógica de negócio e tratamento de erros
- ✅ **Testes de Validação**: Campos obrigatórios e formatos
- ✅ **Testes de Integração**: Cenários de erro e sucesso
- ✅ **Funções Auxiliares**: Dados de teste reutilizáveis

---

## 🔐 Sistema de Gestão de Tokens

O projeto inclui um sistema completo de gestão de tokens para autenticação e recuperação de senha.

### Estrutura do Banco de Dados

```sql
CREATE TABLE user_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    code TEXT NOT NULL,
    token_type TEXT DEFAULT 'reset_password',
    expires_at TIMESTAMP NOT NULL,
    consumed BOOLEAN DEFAULT FALSE,
    dt_created TIMESTAMP NOT NULL DEFAULT now()
);

CREATE INDEX idx_user_tokens_user_id ON user_tokens(user_id);
CREATE INDEX idx_user_tokens_token_type ON user_tokens(token_type);
```

### Tipos de Tokens

| Tipo | Descrição | Expiração | Uso |
|------|-----------|-----------|-----|
| `confirm_email` | Confirmação de email | 1 hora | Cadastro de usuário |
| `reset_password` | Reset de senha | 1 hora | Recuperação de senha |

### Modelo de Dados

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub code: String,
    pub token_type: String,
    pub expires_at: DateTime<Utc>,
    pub consumed: bool,
    pub dt_created: DateTime<Utc>,
}
```

### Repositório de Tokens

O `TokenRepository` fornece métodos para gerenciar tokens:

```rust
pub struct TokenRepository<'a> {
    app_state: &'a AppState,
}

impl<'a> TokenRepository<'a> {
    // Criar token para usuário
    pub async fn create_token(
        &self,
        user_id: Uuid,
        token_type: &str,
    ) -> Result<UserToken, sqlx::Error>

    // Buscar token válido por user_id e token_type
    pub async fn find_valid_token(
        &self,
        user_id: Uuid,
        token_type: &str,
    ) -> Result<Option<UserToken>, sqlx::Error>

    // Buscar token válido por código
    pub async fn find_valid_token_by_code(
        &self,
        code: &str,
        token_type: &str,
    ) -> Result<Option<UserToken>, sqlx::Error>

    // Marcar token como consumido
    pub async fn mark_as_consumed(&self, token_id: Uuid) -> Result<(), sqlx::Error>
}
```

### Fluxos Implementados

#### 1. **Cadastro de Usuário com Confirmação de Email**

```rust
// 1. Criar usuário e perfil
let user = User::new(...);
let profile = Profile::new(user.id);

// 2. Salvar no banco
repository.create_user_with_profile(&user, &profile).await?;

// 3. Criar token de confirmação
let confirm_token = token_repo
    .create_token(user.id, "confirm_email")
    .await?;

// 4. Enviar email (implementação futura)
println!("Token de confirmação: {}", confirm_token.code);
```

#### 2. **Recuperação de Senha**

```rust
// 1. Verificar se usuário existe
let user = repository.find_by_email(&email).await?;

// 2. Criar token de reset
let reset_token = token_repo
    .create_token(user.id, "reset_password")
    .await?;

// 3. Enviar email (implementação futura)
println!("Token de reset: {}", reset_token.code);
```

#### 3. **Alteração de Senha com Token**

```rust
// 1. Validar token
let token = token_repo
    .find_valid_token_by_code(&code, "reset_password")
    .await?
    .ok_or_else(|| AppError::bad_request("Token inválido"))?;

// 2. Alterar senha
repository.update_password(token.user_id, &hashed_password).await?;

// 3. Marcar token como consumido
token_repo.mark_as_consumed(token.id).await?;
```

#### 4. **Confirmação de Email**

```rust
// 1. Validar token
let token = token_repo
    .find_valid_token_by_code(&code, "confirm_email")
    .await?
    .ok_or_else(|| AppError::bad_request("Token inválido"))?;

// 2. Confirmar email
profile_repo.confirm_email(token.user_id).await?;

// 3. Marcar token como consumido
token_repo.mark_as_consumed(token.id).await?;
```

### Endpoints da API

| Método | Endpoint | Descrição |
|--------|----------|-----------|
| `POST` | `/api/v1/auth/register/` | Cadastro com token de confirmação |
| `POST` | `/api/v1/auth/login/` | Login do usuário |
| `POST` | `/api/v1/auth/forgot-password/` | Solicitar reset de senha |
| `POST` | `/api/v1/auth/change-password/` | Alterar senha com token |
| `GET` | `/api/v1/auth/confirm-email/{code}` | Confirmar email com token |

### Segurança

- **Expiração automática**: Tokens expiram em 1 hora
- **Uso único**: Tokens são marcados como consumidos após uso
- **Validação de tipo**: Cada token tem um tipo específico
- **Limpeza automática**: Tokens expirados não são retornados
- **Índices otimizados**: Busca eficiente por user_id e token_type

### Próximas Implementações

- [ ] **Envio de emails**: Integração com serviço de email
- [ ] **Rate limiting**: Limitar tentativas de criação de tokens
- [ ] **Auditoria**: Log de todas as operações com tokens
- [ ] **Notificações**: Webhooks para eventos de token
- [ ] **Expiração configurável**: Diferentes tempos por tipo de token

## Próximos Passos

### 🚀 **Funcionalidades de E-commerce**
1. **Sistema de Categorias**: Organização hierárquica de produtos
2. **Sistema de Imagens**: Upload e gestão de imagens de produtos
3. **Sistema de Variações**: Produtos com diferentes opções (cor, tamanho, etc.)
4. **Sistema de Avaliações**: Comentários e ratings dos clientes
5. **Sistema de Descontos**: Cupons e promoções
6. **Sistema de Carrinho**: Gestão de carrinho de compras
7. **Sistema de Pedidos**: Processamento e gestão de pedidos
8. **Sistema de Pagamentos**: Integração com gateways de pagamento

### 🔧 **Melhorias Técnicas**
1. **Configurar paginação dinâmica** (limit/offset via query params)
2. **Implementar cache** para melhorar performance
3. **Adicionar documentação OpenAPI/Swagger**
4. **Implementar busca full-text** com Elasticsearch
5. **Sistema de notificações** para eventos de produtos
6. **Logs de auditoria** para todas as operações
7. **Rate limiting** para APIs públicas
8. **Métricas e monitoramento** com Prometheus

### 📱 **Frontend e UX**
1. **Interface administrativa** para gestão de produtos
2. **API de busca** com filtros avançados
3. **Sistema de tags** para categorização
4. **Exportação de dados** (CSV, JSON)
5. **Importação em lote** de produtos
6. **Dashboard de métricas** de vendas 
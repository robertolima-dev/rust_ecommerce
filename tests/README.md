# Testes do Rust Template

Este diretório contém os testes automatizados para o Rust Template.

## 📋 Pré-requisitos

1. **Banco de dados PostgreSQL** rodando
2. **SQLx CLI** instalado: `cargo install sqlx-cli`
3. **Variável de ambiente** `DATABASE_URL_TEST` configurada

## ⚙️ Configuração

### 1. Configurar banco de teste

Adicione a variável `DATABASE_URL_TEST` no seu arquivo `.env`:

```bash
# Banco principal
DATABASE_URL=postgresql://user:pass@localhost:5432/rust_template

# Banco de teste (separado)
DATABASE_URL_TEST=postgresql://user:pass@localhost:5432/rust_template_test
```

### 2. Criar banco de teste

```bash
# Criar banco de teste
sqlx database create --database-url "$DATABASE_URL_TEST"

# Executar migrações no banco de teste
sqlx migrate run --database-url "$DATABASE_URL_TEST"
```

## 🧪 Executando os Testes

### Opção 1: Script automatizado (Recomendado)

```bash
./run_tests.sh
```

O script irá:
- ✅ Verificar se `DATABASE_URL_TEST` está configurada
- ✅ Criar banco de teste se não existir
- ✅ Executar migrações
- ✅ Rodar todos os testes
- ✅ Limpar dados de teste

### Opção 2: Execução manual

```bash
# Configurar ambiente
export APP_ENVIRONMENT=testing
export DATABASE_URL_TEST=postgresql://user:pass@localhost:5432/rust_template_test

# Executar testes
cargo test --test user_tests -- --nocapture
```

### Opção 3: Teste específico

```bash
# Executar apenas um teste específico
cargo test test_create_user --test user_tests -- --nocapture

# Executar testes que contenham "login"
cargo test login --test user_tests -- --nocapture
```

## 📊 Testes Disponíveis

### Testes de Autenticação
- `test_create_user` - Testa criação de usuário
- `test_login_user` - Testa login de usuário
- `test_unauthorized_access` - Testa acesso sem token
- `test_invalid_token` - Testa token inválido

### Testes de Usuário
- `test_get_me_with_token` - Testa busca de dados do usuário logado
- `test_update_user` - Testa atualização de dados do usuário
- `test_update_profile` - Testa atualização de perfil
- `test_delete_user` - Testa exclusão de usuário

### Testes de Sistema
- `test_health_check` - Testa endpoint de health check

## 🔧 Estrutura dos Testes

### Setup e Cleanup
Cada teste:
1. **Setup**: Conecta ao banco de teste e executa migrações
2. **Execução**: Roda o teste específico
3. **Cleanup**: Limpa dados de teste

### Helpers
- `setup_test_db()` - Configura banco de teste
- `cleanup_test_db()` - Limpa dados de teste
- `create_test_user_request()` - Cria dados de teste
- `create_test_login_request()` - Cria dados de login

### Isolamento
- Cada teste usa dados únicos (UUIDs gerados)
- Banco de teste separado do banco principal
- Limpeza automática após cada teste

## 🐛 Debugging

### Logs detalhados
```bash
export RUST_LOG=debug
cargo test --test user_tests -- --nocapture
```

### Verificar banco de teste
```bash
# Conectar ao banco de teste
psql "$DATABASE_URL_TEST"

# Verificar tabelas
\dt

# Verificar dados
SELECT * FROM users;
SELECT * FROM profiles;
```

### Teste individual com logs
```bash
cargo test test_create_user --test user_tests -- --nocapture --exact
```

## 📝 Adicionando Novos Testes

1. **Criar função de teste**:
```rust
#[actix_web::test]
async fn test_nova_funcionalidade() {
    let pool = setup_test_db().await;
    let app_state = create_test_app_state(pool.clone());
    
    // ... lógica do teste ...
    
    cleanup_test_db(&pool).await;
}
```

2. **Usar helpers existentes** ou criar novos
3. **Sempre limpar dados** no final do teste

## 🚨 Troubleshooting

### Erro: "DATABASE_URL_TEST não está configurada"
- Verifique se a variável está no `.env`
- Execute: `echo $DATABASE_URL_TEST`

### Erro: "Falha ao conectar ao banco de teste"
- Verifique se PostgreSQL está rodando
- Verifique credenciais na URL
- Teste: `psql "$DATABASE_URL_TEST"`

### Erro: "Falha ao executar migrações"
- Execute manualmente: `sqlx migrate run --database-url "$DATABASE_URL_TEST"`
- Verifique se o banco existe: `sqlx database create --database-url "$DATABASE_URL_TEST"`

### Testes falhando
- Verifique logs com `RUST_LOG=debug`
- Execute teste específico para debug
- Verifique se banco de teste está limpo 
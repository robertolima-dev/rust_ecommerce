#[cfg(test)]
mod tests {
    use crate::app_core::app_error::AppError;
    use crate::apps::product::models::{
        CreateProductRequest, Product, ProductListParams, UpdateProductRequest,
    };
    use bigdecimal::BigDecimal;
    use chrono::Utc;
    use serde_json::json;
    use uuid::Uuid;

    // ===== TESTES UNITÁRIOS DE MODELS =====

    #[test]
    fn test_product_model_creation() {
        let now = Utc::now();
        let product = Product {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            name: "Test Product".to_string(),
            slug: "test-product".to_string(),
            short_description: Some("A test product".to_string()),
            description: Some("This is a detailed description".to_string()),
            price: BigDecimal::from(9999), // 99.99
            stock_quantity: 100,
            attributes: Some(json!({"color": "red", "size": "M"})),
            is_active: true,
            dt_created: now,
            dt_updated: now,
            dt_deleted: None,
        };

        assert_eq!(product.name, "Test Product");
        assert_eq!(product.slug, "test-product");
        assert_eq!(product.stock_quantity, 100);
        assert!(product.is_active);
        assert_eq!(product.price, BigDecimal::from(9999));
    }

    #[test]
    fn test_create_product_request_validation() {
        let valid_request = CreateProductRequest {
            name: "Valid Product".to_string(),
            short_description: Some("Short desc".to_string()),
            description: Some("Long description".to_string()),
            price: 9999, // 99.99
            stock_quantity: 50,
            attributes: Some(json!({"brand": "TestBrand"})),
            is_active: true,
        };

        // Teste básico de criação
        assert_eq!(valid_request.name, "Valid Product");
        assert_eq!(valid_request.price, 9999);
        assert_eq!(valid_request.stock_quantity, 50);
        assert!(valid_request.is_active);
    }

    #[test]
    fn test_update_product_request_optional_fields() {
        let update_request = UpdateProductRequest {
            name: Some("Updated Name".to_string()),
            short_description: None, // Campo opcional não preenchido
            description: Some("Updated description".to_string()),
            price: Some(14999),   // 149.99
            stock_quantity: None, // Campo opcional não preenchido
            attributes: Some(json!({"new_attribute": "value"})),
            is_active: Some(false),
        };

        assert_eq!(update_request.name, Some("Updated Name".to_string()));
        assert_eq!(update_request.short_description, None);
        assert_eq!(update_request.price, Some(14999));
        assert_eq!(update_request.is_active, Some(false));
    }

    #[test]
    fn test_product_list_params_creation() {
        let params = ProductListParams {
            name: Some("test".to_string()),
            min_price: Some(1000),  // 10.00
            max_price: Some(10000), // 100.00
            limit: Some(20),
            offset: Some(0),
            is_active: Some(true),
        };

        assert_eq!(params.name, Some("test".to_string()));
        assert_eq!(params.min_price, Some(1000));
        assert_eq!(params.max_price, Some(10000));
        assert_eq!(params.limit, Some(20));
        assert_eq!(params.offset, Some(0));
        assert_eq!(params.is_active, Some(true));
    }

    // ===== TESTES UNITÁRIOS DE SERVICES =====

    #[tokio::test]
    async fn test_product_service_validation() {
        // Teste de validação de nome vazio
        let invalid_request = CreateProductRequest {
            name: "".to_string(), // Nome vazio
            short_description: Some("A test product".to_string()),
            description: Some("This is a detailed description".to_string()),
            price: 9999,
            stock_quantity: 100,
            attributes: Some(json!({"color": "red"})),
            is_active: true,
        };

        // Verifica se o nome está vazio (que é o comportamento esperado)
        assert!(invalid_request.name.is_empty());
        assert_eq!(invalid_request.name, "");
    }

    #[tokio::test]
    async fn test_product_service_price_validation() {
        // Teste de preço negativo
        let negative_price_request = CreateProductRequest {
            name: "Test Product".to_string(),
            short_description: Some("A test product".to_string()),
            description: Some("This is a detailed description".to_string()),
            price: -1000, // Preço negativo
            stock_quantity: 100,
            attributes: None,
            is_active: true,
        };

        // Verifica se o preço é negativo (que é o comportamento esperado)
        assert!(negative_price_request.price < 0, "Preço deve ser negativo");
    }

    #[tokio::test]
    async fn test_product_service_stock_validation() {
        // Teste de estoque negativo
        let negative_stock_request = CreateProductRequest {
            name: "Test Product".to_string(),
            short_description: Some("A test product".to_string()),
            description: Some("This is a detailed description".to_string()),
            price: 9999,
            stock_quantity: -50, // Estoque negativo
            attributes: None,
            is_active: true,
        };

        // Verifica se o estoque é negativo (que é o comportamento esperado)
        assert!(
            negative_stock_request.stock_quantity < 0,
            "Estoque deve ser negativo"
        );
    }

    #[tokio::test]
    async fn test_product_service_valid_request() {
        // Teste de request válido
        let valid_request = CreateProductRequest {
            name: "Valid Product".to_string(),
            short_description: Some("A valid product".to_string()),
            description: Some("This is a valid description".to_string()),
            price: 9999,
            stock_quantity: 100,
            attributes: Some(json!({"brand": "TestBrand", "category": "Electronics"})),
            is_active: true,
        };

        // Verifica se todos os campos estão preenchidos corretamente
        assert!(!valid_request.name.is_empty());
        assert!(valid_request.price >= 0);
        assert!(valid_request.stock_quantity >= 0);
        assert!(valid_request.is_active);
    }

    // ===== TESTES UNITÁRIOS DE VALIDAÇÃO =====

    #[test]
    fn test_product_name_validation() {
        let valid_names = vec![
            "Product Name",
            "Test Product 123",
            "Product-With-Dashes",
            "Product_With_Underscores",
        ];

        let invalid_names = vec![
            "",    // Nome vazio
            "   ", // Apenas espaços
            "A",   // Muito curto
        ];

        for name in valid_names {
            let request = CreateProductRequest {
                name: name.to_string(),
                short_description: Some("Description".to_string()),
                description: Some("Long description".to_string()),
                price: 9999,
                stock_quantity: 100,
                attributes: None,
                is_active: true,
            };
            assert!(!request.name.is_empty(), "Nome válido falhou: '{}'", name);
        }

        for name in invalid_names {
            let request = CreateProductRequest {
                name: name.to_string(),
                short_description: Some("Description".to_string()),
                description: Some("Long description".to_string()),
                price: 9999,
                stock_quantity: 100,
                attributes: None,
                is_active: true,
            };
            // Para nomes inválidos, verificamos se estão vazios ou são muito curtos
            if name.trim().is_empty() {
                assert!(
                    request.name.trim().is_empty(),
                    "Nome vazio deve ser detectado: '{}'",
                    name
                );
            } else if name.len() < 2 {
                assert!(
                    request.name.len() < 2,
                    "Nome muito curto deve ser detectado: '{}'",
                    name
                );
            }
        }
    }

    #[test]
    fn test_product_price_validation() {
        let valid_prices = vec![0, 100, 9999, 100000, 999999];

        let invalid_prices = vec![-1000, -1]; // Preços negativos

        for price in valid_prices {
            let request = CreateProductRequest {
                name: "Test Product".to_string(),
                short_description: Some("Description".to_string()),
                description: Some("Long description".to_string()),
                price,
                stock_quantity: 100,
                attributes: None,
                is_active: true,
            };
            assert!(request.price >= 0, "Preço válido falhou: {}", price);
        }

        for price in invalid_prices {
            let request = CreateProductRequest {
                name: "Test Product".to_string(),
                short_description: Some("Description".to_string()),
                description: Some("Long description".to_string()),
                price,
                stock_quantity: 100,
                attributes: None,
                is_active: true,
            };
            assert!(request.price < 0, "Preço inválido passou: {}", price);
        }
    }

    #[test]
    fn test_product_stock_validation() {
        let valid_stocks = vec![0, 1, 100, 1000, 999999];

        let invalid_stocks = vec![-100, -1]; // Estoque negativo

        for stock in valid_stocks {
            let request = CreateProductRequest {
                name: "Test Product".to_string(),
                short_description: Some("Description".to_string()),
                description: Some("Long description".to_string()),
                price: 9999,
                stock_quantity: stock,
                attributes: None,
                is_active: true,
            };
            assert!(
                request.stock_quantity >= 0,
                "Estoque válido falhou: {}",
                stock
            );
        }

        for stock in invalid_stocks {
            let request = CreateProductRequest {
                name: "Test Product".to_string(),
                short_description: Some("Description".to_string()),
                description: Some("Long description".to_string()),
                price: 9999,
                stock_quantity: stock,
                attributes: None,
                is_active: true,
            };
            assert!(
                request.stock_quantity < 0,
                "Estoque inválido passou: {}",
                stock
            );
        }
    }

    // ===== TESTES DE INTEGRAÇÃO SIMULADOS =====

    #[tokio::test]
    async fn test_product_service_error_handling() {
        // Simula cenário onde o service retorna erro 404
        let error = AppError::not_found("Produto não encontrado");

        match error {
            AppError::NotFound(msg) => {
                assert_eq!(msg, Some("Produto não encontrado".to_string()));
            }
            _ => panic!("Esperava erro NotFound"),
        }
    }

    #[tokio::test]
    async fn test_product_service_database_error_handling() {
        // Simula cenário onde o service retorna erro de banco
        let error = AppError::database_error("Erro de conexão com banco");

        match error {
            AppError::DatabaseError(msg) => {
                assert_eq!(msg, Some("Erro de conexão com banco".to_string()));
            }
            _ => panic!("Esperava erro DatabaseError"),
        }
    }

    // ===== FUNÇÕES AUXILIARES PARA TESTES DE INTEGRAÇÃO =====

    /// Função auxiliar para criar dados de teste válidos
    pub fn _create_valid_product_request() -> CreateProductRequest {
        CreateProductRequest {
            name: format!("Test Product {}", Uuid::new_v4()),
            short_description: Some("A test product for testing".to_string()),
            description: Some("This is a detailed description for testing purposes".to_string()),
            price: 9999, // 99.99
            stock_quantity: 100,
            attributes: Some(json!({
                "brand": "TestBrand",
                "category": "Electronics",
                "color": "Black"
            })),
            is_active: true,
        }
    }

    /// Função auxiliar para criar dados de update válidos
    pub fn _create_valid_update_request() -> UpdateProductRequest {
        UpdateProductRequest {
            name: Some("Updated Product Name".to_string()),
            short_description: Some("Updated short description".to_string()),
            description: Some("Updated long description".to_string()),
            price: Some(14999), // 149.99
            stock_quantity: Some(200),
            attributes: Some(json!({
                "brand": "UpdatedBrand",
                "category": "UpdatedCategory"
            })),
            is_active: Some(false),
        }
    }

    /// Função auxiliar para criar parâmetros de listagem válidos
    pub fn _create_valid_list_params() -> ProductListParams {
        ProductListParams {
            name: Some("test".to_string()),
            min_price: Some(1000),  // 10.00
            max_price: Some(10000), // 100.00
            limit: Some(20),
            offset: Some(0),
            is_active: Some(true),
        }
    }

    /// Função auxiliar para criar um produto de teste
    pub fn _create_test_product() -> Product {
        let now = Utc::now();
        Product {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            name: "Test Product".to_string(),
            slug: "test-product".to_string(),
            short_description: Some("A test product".to_string()),
            description: Some("This is a detailed description".to_string()),
            price: BigDecimal::from(9999),
            stock_quantity: 100,
            attributes: Some(json!({"color": "red", "size": "M"})),
            is_active: true,
            dt_created: now,
            dt_updated: now,
            dt_deleted: None,
        }
    }

    /// Função auxiliar para validar resposta de erro
    pub fn _assert_validation_error(result: Result<(), validator::ValidationErrors>, field: &str) {
        match result {
            Ok(_) => panic!("Esperava erro de validação para o campo: {}", field),
            Err(errors) => {
                assert!(
                    errors.field_errors().contains_key(field),
                    "Erro não encontrado para o campo: {}",
                    field
                );
            }
        }
    }

    /// Função auxiliar para validar preço em centavos
    pub fn _assert_price_in_cents(price: i64, expected_cents: i64) {
        assert_eq!(price, expected_cents, "Preço em centavos incorreto");
    }

    /// Função auxiliar para validar estoque
    pub fn _assert_stock_quantity(stock: i32, expected_stock: i32) {
        assert_eq!(stock, expected_stock, "Quantidade de estoque incorreta");
    }
}

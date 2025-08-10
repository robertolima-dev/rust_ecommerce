#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::app_core::app_error::AppError;
    // use crate::app_core::app_state::AppState;
    // use crate::apps::user::services::UserService;
    use crate::apps::user::models::{LoginRequest, User, UserRequest};
    use uuid::Uuid;
    use validator::Validate;

    // ===== TESTES UNITÁRIOS DE SERVICES =====

    #[tokio::test]
    async fn test_user_service_validation() {
        // Teste de validação de email inválido
        let invalid_request = UserRequest {
            email: "invalid-email".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            password: "password123".to_string(),
            profile: None,
            tenant_id: None,
        };

        // Simular validação (sem banco de dados)
        let validation_result = invalid_request.validate();
        assert!(validation_result.is_err());
    }

    #[tokio::test]
    async fn test_user_service_password_validation() {
        // Teste de senha muito curta
        let weak_password_request = UserRequest {
            email: "test@example.com".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            password: "123".to_string(), // Senha muito curta
            profile: None,
            tenant_id: None,
        };

        let validation_result = weak_password_request.validate();
        assert!(validation_result.is_err());
    }

    #[tokio::test]
    async fn test_user_service_valid_request() {
        // Teste de request válido
        let valid_request = UserRequest {
            email: "test@example.com".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            password: "password123".to_string(),
            profile: None,
            tenant_id: None,
        };

        let validation_result = valid_request.validate();
        assert!(validation_result.is_ok());
    }

    // ===== TESTES UNITÁRIOS DE MODELS =====

    #[test]
    fn test_user_model_creation() {
        let user = User::new(
            "testuser",
            "test@example.com",
            "Test",
            "User",
            "password123",
        )
        .expect("Deve criar usuário válido");

        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.first_name, "Test");
        assert_eq!(user.last_name, "User");
    }

    #[test]
    fn test_user_password_verification() {
        let user = User::new(
            "testuser",
            "test@example.com",
            "Test",
            "User",
            "password123",
        )
        .expect("Deve criar usuário válido");

        // Senha correta
        assert!(user.verify_password("password123"));

        // Senha incorreta
        assert!(!user.verify_password("wrongpassword"));
    }

    // ===== TESTES UNITÁRIOS DE VALIDAÇÃO =====

    #[test]
    fn test_email_validation() {
        let valid_emails = vec![
            "test@example.com",
            "user.name@domain.co.uk",
            "user+tag@example.org",
        ];

        let invalid_emails = vec!["invalid-email", "@example.com", "user@", "user@.com"];

        for email in valid_emails {
            let request = LoginRequest {
                email: email.to_string(),
                password: "password123".to_string(),
            };
            assert!(request.validate().is_ok(), "Email válido falhou: {}", email);
        }

        for email in invalid_emails {
            let request = LoginRequest {
                email: email.to_string(),
                password: "password123".to_string(),
            };
            assert!(
                request.validate().is_err(),
                "Email inválido passou: {}",
                email
            );
        }
    }

    #[test]
    fn test_password_validation() {
        let valid_passwords = vec!["password123", "MyPass123", "Secure@123"];

        let invalid_passwords = vec![
            "123",      // Muito curta
            "password", // Sem números
            "12345678", // Sem letras
            "pass",     // Muito curta
        ];

        for password in valid_passwords {
            let request = LoginRequest {
                email: "test@example.com".to_string(),
                password: password.to_string(),
            };
            assert!(
                request.validate().is_ok(),
                "Senha válida falhou: {}",
                password
            );
        }

        for password in invalid_passwords {
            let request = LoginRequest {
                email: "test@example.com".to_string(),
                password: password.to_string(),
            };
            assert!(
                request.validate().is_err(),
                "Senha inválida passou: {}",
                password
            );
        }
    }

    // ===== FUNÇÕES AUXILIARES PARA TESTES DE INTEGRAÇÃO =====

    /// Função auxiliar para criar dados de teste válidos
    pub fn _create_valid_user_request() -> UserRequest {
        UserRequest {
            email: format!("test_{}@example.com", Uuid::new_v4()),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            password: "password123".to_string(),
            profile: None,
            tenant_id: None,
        }
    }

    /// Função auxiliar para criar dados de login válidos
    pub fn _create_valid_login_request(email: &str) -> LoginRequest {
        LoginRequest {
            email: email.to_string(),
            password: "password123".to_string(),
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
}

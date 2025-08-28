use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiToken {
    pub token: String,
    pub user_id: String,
    pub permissions: Vec<Permission>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Permission {
    Read,
    Write,
    Admin,
    LfsUpload,
    LfsDownload,
    Lock,
    Unlock,
}

#[derive(Debug, Clone)]
pub struct AuthService {
    tokens: HashMap<String, ApiToken>,
    server_tokens: HashMap<String, String>, // server_id -> token
}

impl AuthService {
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
            server_tokens: HashMap::new(),
        }
    }

    pub fn generate_token(&mut self, user_id: String, permissions: Vec<Permission>) -> Result<String> {
        let token = format!("rune_{}_{}", user_id, chrono::Utc::now().timestamp());
        let api_token = ApiToken {
            token: token.clone(),
            user_id,
            permissions,
            expires_at: Some(chrono::Utc::now() + chrono::Duration::days(30)),
            created_at: chrono::Utc::now(),
        };
        
        self.tokens.insert(token.clone(), api_token);
        Ok(token)
    }

    pub fn generate_server_token(&mut self, server_id: String) -> Result<String> {
        let token = self.generate_token(
            format!("server_{}", server_id),
            vec![Permission::Read, Permission::Write, Permission::LfsUpload, Permission::LfsDownload]
        )?;
        self.server_tokens.insert(server_id, token.clone());
        Ok(token)
    }

    pub fn validate_token(&self, token: &str) -> Option<&ApiToken> {
        let api_token = self.tokens.get(token)?;
        
        // Check if token is expired
        if let Some(expires_at) = api_token.expires_at {
            if chrono::Utc::now() > expires_at {
                return None;
            }
        }
        
        Some(api_token)
    }

    pub fn has_permission(&self, token: &str, permission: Permission) -> bool {
        if let Some(api_token) = self.validate_token(token) {
            api_token.permissions.contains(&permission) || api_token.permissions.contains(&Permission::Admin)
        } else {
            false
        }
    }

    pub fn revoke_token(&mut self, token: &str) -> bool {
        self.tokens.remove(token).is_some()
    }

    pub fn list_tokens(&self) -> Vec<&ApiToken> {
        self.tokens.values().collect()
    }
}

impl Default for AuthService {
    fn default() -> Self {
        Self::new()
    }
}

// Middleware for authentication
pub async fn auth_middleware(
    State(auth): State<std::sync::Arc<std::sync::Mutex<AuthService>>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract token from Authorization header
    let token = headers
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate token
    let auth_guard = auth.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if auth_guard.validate_token(token).is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    drop(auth_guard);

    // Continue to next handler
    Ok(next.run(request).await)
}

// Middleware for specific permissions
pub fn require_permission(permission: Permission) -> impl Fn(State<std::sync::Arc<std::sync::Mutex<AuthService>>>, HeaderMap, Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>> + Clone {
    move |State(auth): State<std::sync::Arc<std::sync::Mutex<AuthService>>>, headers: HeaderMap, request: Request, next: Next| {
        let perm = permission.clone();
        Box::pin(async move {
            // Extract token from Authorization header
            let token = headers
                .get("Authorization")
                .and_then(|header| header.to_str().ok())
                .and_then(|header| header.strip_prefix("Bearer "))
                .ok_or(StatusCode::UNAUTHORIZED)?;

            // Check permission
            let has_permission = {
                let auth_guard = auth.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                auth_guard.has_permission(token, perm)
            }; // Drop the lock here
            
            if !has_permission {
                return Err(StatusCode::FORBIDDEN);
            }

            // Continue to next handler
            Ok(next.run(request).await)
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct CreateTokenRequest {
    pub user_id: String,
    pub permissions: Vec<Permission>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateTokenResponse {
    pub token: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_service_creation() {
        let auth = AuthService::new();
        assert_eq!(auth.tokens.len(), 0);
        assert_eq!(auth.server_tokens.len(), 0);
    }

    #[test]
    fn test_token_generation() {
        let mut auth = AuthService::new();
        let token = auth.generate_token(
            "test_user".to_string(),
            vec![Permission::Read, Permission::Write]
        ).unwrap();
        
        assert!(!token.is_empty());
        assert!(token.starts_with("rune_test_user_"));
        assert!(auth.validate_token(&token).is_some());
    }

    #[test]
    fn test_server_token_generation() {
        let mut auth = AuthService::new();
        let token = auth.generate_server_token("server1".to_string()).unwrap();
        
        assert!(!token.is_empty());
        assert!(auth.validate_token(&token).is_some());
        assert!(auth.has_permission(&token, Permission::Read));
        assert!(auth.has_permission(&token, Permission::Write));
    }

    #[test]
    fn test_permission_checking() {
        let mut auth = AuthService::new();
        let token = auth.generate_token(
            "test_user".to_string(),
            vec![Permission::Read]
        ).unwrap();
        
        assert!(auth.has_permission(&token, Permission::Read));
        assert!(!auth.has_permission(&token, Permission::Write));
        assert!(!auth.has_permission(&token, Permission::Admin));
    }

    #[test]
    fn test_admin_permission() {
        let mut auth = AuthService::new();
        let token = auth.generate_token(
            "admin_user".to_string(),
            vec![Permission::Admin]
        ).unwrap();
        
        // Admin should have all permissions
        assert!(auth.has_permission(&token, Permission::Read));
        assert!(auth.has_permission(&token, Permission::Write));
        assert!(auth.has_permission(&token, Permission::LfsUpload));
        assert!(auth.has_permission(&token, Permission::Admin));
    }

    #[test]
    fn test_token_revocation() {
        let mut auth = AuthService::new();
        let token = auth.generate_token(
            "test_user".to_string(),
            vec![Permission::Read]
        ).unwrap();
        
        assert!(auth.validate_token(&token).is_some());
        assert!(auth.revoke_token(&token));
        assert!(auth.validate_token(&token).is_none());
    }
}

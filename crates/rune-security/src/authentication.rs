use anyhow::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub mfa_enabled: bool,
    pub totp_secret: Option<String>,
    pub backup_codes: Vec<String>,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub session_timeout: u64,
    pub max_login_attempts: u32,
    pub password_min_length: usize,
    pub require_mfa: bool,
    pub totp_issuer: String,
    pub jwt_secret: String,
    pub password_policy: PasswordPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_symbols: bool,
    pub max_age_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub ip_address: String,
    pub user_agent: String,
}

pub struct AuthenticationManager {
    users: HashMap<String, User>,
    sessions: HashMap<String, Session>,
    failed_attempts: HashMap<String, u32>,
    config: AuthConfig,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            session_timeout: 3600, // 1 hour
            max_login_attempts: 5,
            password_min_length: 8,
            require_mfa: false,
            totp_issuer: "Rune VCS".to_string(),
            jwt_secret: "your-secret-key".to_string(),
            password_policy: PasswordPolicy::default(),
        }
    }
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_symbols: false,
            max_age_days: Some(90),
        }
    }
}

impl AuthenticationManager {
    pub fn new(config: AuthConfig) -> Self {
        Self {
            users: HashMap::new(),
            sessions: HashMap::new(),
            failed_attempts: HashMap::new(),
            config,
        }
    }

    pub fn create_user(&mut self, username: String, email: String, _password: &str) -> Result<String> {
        let user_id = uuid::Uuid::new_v4().to_string();
        let password_hash = bcrypt::hash(_password, bcrypt::DEFAULT_COST)?;
        
        let user = User {
            id: user_id.clone(),
            username,
            email,
            password_hash,
            created_at: Utc::now(),
            last_login: None,
            is_active: true,
            mfa_enabled: false,
            totp_secret: None,
            backup_codes: Vec::new(),
            roles: Vec::new(),
            permissions: Vec::new(),
        };
        
        self.users.insert(user_id.clone(), user);
        Ok(user_id)
    }

    pub fn authenticate_user(&mut self, username: &str, _password: &str) -> Result<Option<String>> {
        // Find user by username
        let user_id = self.users.iter()
            .find(|(_, user)| user.username == username)
            .map(|(id, _)| id.clone());
        
        if let Some(id) = user_id {
            // In a real implementation, verify password hash here
            Ok(Some(id))
        } else {
            Ok(None)
        }
    }

    pub fn setup_totp(&mut self, user_id: &str) -> Result<String> {
        let secret = self.generate_totp_secret();
        if let Some(user) = self.users.get_mut(user_id) {
            user.totp_secret = Some(secret.clone());
            user.mfa_enabled = true;
            
            // Generate QR code URL
            let qr_url = format!(
                "otpauth://totp/{}:{}?secret={}&issuer={}",
                self.config.totp_issuer,
                user.username,
                secret,
                self.config.totp_issuer
            );
            
            Ok(qr_url)
        } else {
            Err(anyhow::anyhow!("User not found"))
        }
    }

    pub fn verify_totp(&self, user_id: &str, token: &str) -> Result<bool> {
        if let Some(user) = self.users.get(user_id) {
            if let Some(_secret) = &user.totp_secret {
                // In a real implementation, verify TOTP token here
                Ok(token.len() == 6 && token.chars().all(|c| c.is_ascii_digit()))
            } else {
                Ok(false)
            }
        } else {
            Err(anyhow::anyhow!("User not found"))
        }
    }

    pub fn generate_backup_codes(&mut self, user_id: &str) -> Result<Vec<String>> {
        let mut codes = Vec::new();
        for _ in 0..8 {
            codes.push(self.generate_backup_code());
        }
        
        if let Some(user) = self.users.get_mut(user_id) {
            user.backup_codes = codes.clone();
            Ok(codes)
        } else {
            Err(anyhow::anyhow!("User not found"))
        }
    }

    pub fn create_session(&mut self, user_id: &str, ip_address: String, user_agent: String) -> Result<String> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = now + chrono::Duration::seconds(self.config.session_timeout as i64);
        
        let session = Session {
            session_id: session_id.clone(),
            user_id: user_id.to_string(),
            created_at: now,
            expires_at,
            last_activity: now,
            ip_address,
            user_agent,
        };
        
        self.sessions.insert(session_id.clone(), session);
        Ok(session_id)
    }

    pub fn validate_session(&mut self, session_id: &str) -> Result<bool> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            let now = Utc::now();
            if now > session.expires_at {
                self.sessions.remove(session_id);
                Ok(false)
            } else {
                session.last_activity = now;
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    pub fn revoke_session(&mut self, session_id: &str) -> Result<()> {
        self.sessions.remove(session_id);
        Ok(())
    }

    fn generate_totp_secret(&self) -> String {
        let secret: [u8; 20] = rand::random();
        base32::encode(base32::Alphabet::RFC4648 { padding: false }, &secret)
    }

    fn generate_backup_code(&self) -> String {
        let code: u32 = rand::random::<u32>() % 100000000; // 8 digits
        hex::encode(code.to_be_bytes())[..8].to_string()
    }

    pub fn check_password_policy(&self, password: &str) -> Result<()> {
        let policy = &self.config.password_policy;
        
        if password.len() < policy.min_length {
            return Err(anyhow::anyhow!("Password too short"));
        }
        
        if policy.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err(anyhow::anyhow!("Password must contain uppercase letters"));
        }
        
        if policy.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return Err(anyhow::anyhow!("Password must contain lowercase letters"));
        }
        
        if policy.require_numbers && !password.chars().any(|c| c.is_ascii_digit()) {
            return Err(anyhow::anyhow!("Password must contain numbers"));
        }
        
        if policy.require_symbols && !password.chars().any(|c| !c.is_alphanumeric()) {
            return Err(anyhow::anyhow!("Password must contain symbols"));
        }
        
        Ok(())
    }

    pub fn get_user(&self, user_id: &str) -> Option<&User> {
        self.users.get(user_id)
    }

    pub fn update_user_roles(&mut self, user_id: &str, roles: Vec<String>) -> Result<()> {
        if let Some(user) = self.users.get_mut(user_id) {
            user.roles = roles;
            Ok(())
        } else {
            Err(anyhow::anyhow!("User not found"))
        }
    }

    pub fn add_user_permission(&mut self, user_id: &str, permission: String) -> Result<()> {
        if let Some(user) = self.users.get_mut(user_id) {
            if !user.permissions.contains(&permission) {
                user.permissions.push(permission);
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("User not found"))
        }
    }

    pub fn remove_user_permission(&mut self, user_id: &str, permission: &str) -> Result<()> {
        if let Some(user) = self.users.get_mut(user_id) {
            user.permissions.retain(|p| p != permission);
            Ok(())
        } else {
            Err(anyhow::anyhow!("User not found"))
        }
    }

    pub fn check_permission(&self, user_id: &str, permission: &str) -> bool {
        if let Some(user) = self.users.get(user_id) {
            user.permissions.contains(&permission.to_string())
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let mut auth_manager = AuthenticationManager::new(AuthConfig::default());
        let user_id = auth_manager.create_user(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "password123"
        ).unwrap();
        
        assert!(!user_id.is_empty());
        assert!(auth_manager.get_user(&user_id).is_some());
    }

    #[test]
    fn test_password_policy() {
        let auth_manager = AuthenticationManager::new(AuthConfig::default());
        
        assert!(auth_manager.check_password_policy("Pass123").is_ok());
        assert!(auth_manager.check_password_policy("weak").is_err());
        assert!(auth_manager.check_password_policy("nouppercase123").is_err());
    }

    #[test]
    fn test_session_management() {
        let mut auth_manager = AuthenticationManager::new(AuthConfig::default());
        let user_id = auth_manager.create_user(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "password123"
        ).unwrap();
        
        let session_id = auth_manager.create_session(
            &user_id,
            "127.0.0.1".to_string(),
            "test-agent".to_string()
        ).unwrap();
        
        assert!(auth_manager.validate_session(&session_id).unwrap());
        
        auth_manager.revoke_session(&session_id).unwrap();
        assert!(!auth_manager.validate_session(&session_id).unwrap());
    }
}

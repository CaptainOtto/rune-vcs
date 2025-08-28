use anyhow::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub event_type: AuditEventType,
    pub user_id: Option<String>,
    pub resource: String,
    pub action: String,
    pub result: AuditResult,
    pub timestamp: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub additional_data: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    Authentication,
    Authorization,
    DataAccess,
    DataModification,
    SystemAccess,
    ConfigurationChange,
    SecurityEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Failure,
    Blocked,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub enabled: bool,
    pub log_level: AuditLogLevel,
    pub retention_days: u32,
    pub export_enabled: bool,
    pub real_time_monitoring: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditLogLevel {
    All,
    SecurityOnly,
    Critical,
    Minimal,
}

pub struct AuditManager {
    config: AuditConfig,
    events: Vec<AuditEvent>,
}

impl AuditManager {
    pub fn new(config: AuditConfig) -> Self {
        Self {
            config,
            events: Vec::new(),
        }
    }

    pub fn log_event(&mut self, event: AuditEvent) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        if self.should_log_event(&event) {
            self.events.push(event.clone());
            
            if self.config.real_time_monitoring {
                self.process_real_time_alert(&event)?;
            }
        }
        
        Ok(())
    }

    pub fn log_authentication(&mut self, user_id: Option<String>, result: AuditResult, ip_address: Option<String>) -> Result<()> {
        let event = AuditEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: AuditEventType::Authentication,
            user_id,
            resource: "auth_system".to_string(),
            action: "login".to_string(),
            result,
            timestamp: Utc::now(),
            ip_address,
            user_agent: None,
            additional_data: HashMap::new(),
        };
        
        self.log_event(event)
    }

    pub fn log_data_access(&mut self, user_id: String, resource: String, action: String) -> Result<()> {
        let event = AuditEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: AuditEventType::DataAccess,
            user_id: Some(user_id),
            resource,
            action,
            result: AuditResult::Success,
            timestamp: Utc::now(),
            ip_address: None,
            user_agent: None,
            additional_data: HashMap::new(),
        };
        
        self.log_event(event)
    }

    pub fn log_security_event(&mut self, description: String, severity: AuditResult) -> Result<()> {
        let mut additional_data = HashMap::new();
        additional_data.insert("description".to_string(), description);
        
        let event = AuditEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: AuditEventType::SecurityEvent,
            user_id: None,
            resource: "security_system".to_string(),
            action: "security_alert".to_string(),
            result: severity,
            timestamp: Utc::now(),
            ip_address: None,
            user_agent: None,
            additional_data,
        };
        
        self.log_event(event)
    }

    pub fn get_events_by_user(&self, user_id: &str, limit: Option<usize>) -> Vec<&AuditEvent> {
        let mut events: Vec<&AuditEvent> = self.events
            .iter()
            .filter(|e| e.user_id.as_deref() == Some(user_id))
            .collect();
        
        events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        if let Some(limit) = limit {
            events.truncate(limit);
        }
        
        events
    }

    pub fn get_security_events(&self, hours: u32) -> Vec<&AuditEvent> {
        let cutoff = Utc::now() - chrono::Duration::hours(hours as i64);
        
        self.events
            .iter()
            .filter(|e| {
                matches!(e.event_type, AuditEventType::SecurityEvent) && e.timestamp > cutoff
            })
            .collect()
    }

    pub fn export_events(&self, format: ExportFormat) -> Result<String> {
        match format {
            ExportFormat::JSON => {
                Ok(serde_json::to_string_pretty(&self.events)?)
            }
            ExportFormat::CSV => {
                let mut csv = String::from("ID,Type,User,Resource,Action,Result,Timestamp,IP\n");
                for event in &self.events {
                    csv.push_str(&format!(
                        "{},{:?},{},{},{},{:?},{},{}\n",
                        event.id,
                        event.event_type,
                        event.user_id.as_deref().unwrap_or(""),
                        event.resource,
                        event.action,
                        event.result,
                        event.timestamp.format("%Y-%m-%d %H:%M:%S"),
                        event.ip_address.as_deref().unwrap_or("")
                    ));
                }
                Ok(csv)
            }
        }
    }

    pub fn cleanup_old_events(&mut self) -> Result<usize> {
        let cutoff = Utc::now() - chrono::Duration::days(self.config.retention_days as i64);
        let original_count = self.events.len();
        
        self.events.retain(|event| event.timestamp > cutoff);
        
        Ok(original_count - self.events.len())
    }

    fn should_log_event(&self, event: &AuditEvent) -> bool {
        match self.config.log_level {
            AuditLogLevel::All => true,
            AuditLogLevel::SecurityOnly => {
                matches!(event.event_type, AuditEventType::SecurityEvent | AuditEventType::Authentication)
            }
            AuditLogLevel::Critical => {
                matches!(event.result, AuditResult::Failure | AuditResult::Blocked)
            }
            AuditLogLevel::Minimal => {
                matches!(event.event_type, AuditEventType::SecurityEvent) &&
                matches!(event.result, AuditResult::Failure | AuditResult::Blocked)
            }
        }
    }

    fn process_real_time_alert(&self, event: &AuditEvent) -> Result<()> {
        // TODO: Implement real-time alerting (email, Slack, webhook, etc.)
        if matches!(event.result, AuditResult::Failure | AuditResult::Blocked) {
            println!("SECURITY ALERT: {:?} - {} on {}", event.result, event.action, event.resource);
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum ExportFormat {
    JSON,
    CSV,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_level: AuditLogLevel::All,
            retention_days: 90,
            export_enabled: true,
            real_time_monitoring: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_logging() {
        let config = AuditConfig::default();
        let mut audit_manager = AuditManager::new(config);
        
        audit_manager.log_authentication(
            Some("user123".to_string()),
            AuditResult::Success,
            Some("192.168.1.1".to_string())
        ).unwrap();
        
        assert_eq!(audit_manager.events.len(), 1);
        
        let events = audit_manager.get_events_by_user("user123", None);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_security_event_filtering() {
        let config = AuditConfig::default();
        let mut audit_manager = AuditManager::new(config);
        
        audit_manager.log_security_event(
            "Suspicious login attempt".to_string(),
            AuditResult::Blocked
        ).unwrap();
        
        let security_events = audit_manager.get_security_events(24);
        assert_eq!(security_events.len(), 1);
    }

    #[test]
    fn test_export_events() {
        let config = AuditConfig::default();
        let mut audit_manager = AuditManager::new(config);
        
        audit_manager.log_authentication(
            Some("user123".to_string()),
            AuditResult::Success,
            Some("192.168.1.1".to_string())
        ).unwrap();
        
        let json_export = audit_manager.export_events(ExportFormat::JSON).unwrap();
        assert!(json_export.contains("user123"));
        
        let csv_export = audit_manager.export_events(ExportFormat::CSV).unwrap();
        assert!(csv_export.contains("user123"));
    }
}

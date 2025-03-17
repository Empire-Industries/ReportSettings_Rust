use ::serde::*;
use sendgrid::v3::Email;
use ssql::prelude::tiberius::{AuthMethod, Config, EncryptionLevel};
use std::env;

#[cfg(test)]
mod tests {
    use super::*;

    // Mock the env variable used in the get_settings method for testing
    fn mock_env_variable() {
        env::set_var(
            "SecretBlob",
            r#"{
                "DatabaseServer": "localhost",
                "DatabaseName": "test_db",
                "DatabaseUsername": "admin",
                "DatabasePassword": "password123",
                "LogWebhookUri": "https://example.com",
                "SendgridApiKey": "sendgrid-api-key",
                "EmailFromName": "Test",
                "EmailFromAddress": "test@example.com",
                "EmailToAddresses": "user1@example.com,user2@example.com"
            }"#,
        );
    }

    #[test]
    fn test_get_settings_success() {
        mock_env_variable();

        let result = Settings::get_settings();
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.database_server, "localhost");
        assert_eq!(settings.database_name, "test_db");
    }

    #[test]
    fn test_get_settings_missing_env_var() {
        env::remove_var("SecretBlob");

        let result = Settings::get_settings();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Error getting env variable: environment variable not found");
    }

    #[test]
    fn test_get_settings_invalid_json() {
        env::set_var("SecretBlob", "invalid json");

        let result = Settings::get_settings();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Could not deserialize settings blob: expected value at line 1 column 1"));
    }

    #[test]
    fn test_get_sql_settings() {
        let settings = Settings {
            database_server: "localhost".to_string(),
            database_name: "test_db".to_string(),
            database_username: "admin".to_string(),
            database_password: "password123".to_string(),
            log_webhook_uri: "http://example.com".to_string(),
            sendgrid_api_key: "sendgrid-api-key".to_string(),
            email_from_name: "Test".to_string(),
            email_from_address: "test@example.com".to_string(),
            email_to_addresses: "user1@example.com".to_string(),
        };
    
        let sql_settings = settings.get_sql_settings();
    
        assert_eq!(sql_settings.get_addr(), "localhost:1433");
    }
    
    #[test]
    fn test_get_email_destinations() {
        let settings = Settings {
            database_server: "localhost".to_string(),
            database_name: "test_db".to_string(),
            database_username: "admin".to_string(),
            database_password: "password123".to_string(),
            log_webhook_uri: "https://example.com".to_string(),
            sendgrid_api_key: "sendgrid-api-key".to_string(),
            email_from_name: "Test".to_string(),
            email_from_address: "test@example.com".to_string(),
            email_to_addresses: "user1@example.com,user2@example.com".to_string(),
        };
    
        let email_destinations = settings.get_email_destinations();
    
        assert_eq!(email_destinations.len(), 2);
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Settings {
    database_server: String,
    database_name: String,
    database_username: String,
    database_password: String,
    log_webhook_uri: String,
    sendgrid_api_key: String,
    email_from_name: String,
    email_from_address: String,
    email_to_addresses: String,
}

impl Settings {
    pub fn get_settings() -> Result<Settings, String> {
        let secret_blob = match env::var("SecretBlob") {
            Ok(s) => s,
            Err(e) => return Err(format!("Error getting env variable: {}", e.to_string())),
        };

        let sett: Settings = match serde_json::from_str(&secret_blob) {
            Ok(s) => s,
            Err(e) => {
                return Err(format!(
                    "Could not deserialize settings blob: {}",
                    e.to_string()
                ))
            }
        };

        Ok(sett)
    }

    pub fn get_sql_settings(&self) -> Config {
        let mut sql_settings = Config::new();
        sql_settings.host(&self.database_server);
        sql_settings.application_name("Login Checker");
        sql_settings.database(&self.database_name);
        sql_settings.authentication(AuthMethod::sql_server(
            &self.database_username,
            &self.database_password,
        ));
        sql_settings.encryption(EncryptionLevel::Off);
        sql_settings.trust_cert();
        sql_settings
    }

    pub fn get_email_destinations(&self) -> Vec<Email> {
        self.email_to_addresses
            .split(",")
            .map(|x| Email::new(x))
            .collect()
    }
}

//! SSH profile management for AgTerm
//!
//! This module provides SSH connection profile management with support for:
//! - Profile creation and management
//! - SSH config file parsing
//! - SSH command generation with various options

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, warn};

/// SSH connection profile
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SshProfile {
    /// Profile name (user-defined)
    pub name: String,
    /// Remote host (hostname or IP)
    pub host: String,
    /// SSH port (default: 22)
    #[serde(default = "default_ssh_port")]
    pub port: u16,
    /// Username for SSH connection
    pub user: Option<String>,
    /// Path to identity file (private key)
    pub identity_file: Option<PathBuf>,
    /// Enable SSH agent forwarding
    #[serde(default)]
    pub forward_agent: bool,
    /// ProxyJump host for SSH tunneling
    pub proxy_jump: Option<String>,
    /// Additional SSH options (e.g., "StrictHostKeyChecking=no")
    #[serde(default)]
    pub extra_options: Vec<String>,
}

fn default_ssh_port() -> u16 {
    22
}

impl SshProfile {
    /// Create a new SSH profile with minimal required information
    pub fn new(name: String, host: String) -> Self {
        Self {
            name,
            host,
            port: 22,
            user: None,
            identity_file: None,
            forward_agent: false,
            proxy_jump: None,
            extra_options: Vec::new(),
        }
    }

    /// Generate SSH command arguments from profile
    pub fn to_command(&self) -> Vec<String> {
        let mut args = vec!["ssh".to_string()];

        // Username
        if let Some(user) = &self.user {
            args.push("-l".to_string());
            args.push(user.clone());
        }

        // Port
        if self.port != 22 {
            args.push("-p".to_string());
            args.push(self.port.to_string());
        }

        // Identity file
        if let Some(identity) = &self.identity_file {
            args.push("-i".to_string());
            args.push(identity.display().to_string());
        }

        // Agent forwarding
        if self.forward_agent {
            args.push("-A".to_string());
        }

        // Proxy jump
        if let Some(proxy) = &self.proxy_jump {
            args.push("-J".to_string());
            args.push(proxy.clone());
        }

        // Extra options
        for opt in &self.extra_options {
            args.push("-o".to_string());
            args.push(opt.clone());
        }

        // Host (must be last)
        args.push(self.host.clone());

        args
    }

    /// Parse SSH config file entry into a profile
    /// Returns None if hostname not found or parsing fails
    pub fn from_ssh_config(hostname: &str) -> Option<Self> {
        let config_path = ssh_config_path()?;
        if !config_path.exists() {
            debug!("SSH config file not found at {:?}", config_path);
            return None;
        }

        let content = fs::read_to_string(&config_path).ok()?;
        let parsed = parse_ssh_config(&content);

        if let Some(entry) = parsed.get(hostname) {
            Some(SshProfile {
                name: hostname.to_string(),
                host: entry
                    .get("hostname")
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| hostname.to_string()),
                port: entry
                    .get("port")
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or(22),
                user: entry.get("user").map(|s| s.to_string()),
                identity_file: entry.get("identityfile").map(|s| {
                    let expanded = shellexpand::tilde(s);
                    PathBuf::from(expanded.as_ref())
                }),
                forward_agent: entry
                    .get("forwardagent")
                    .map(|v| v.to_lowercase() == "yes")
                    .unwrap_or(false),
                proxy_jump: entry.get("proxyjump").map(|s| s.to_string()),
                extra_options: Vec::new(),
            })
        } else {
            debug!("Host '{}' not found in SSH config", hostname);
            None
        }
    }

    /// Get the connection string for display (e.g., "user@host:port")
    pub fn connection_string(&self) -> String {
        let user_part = self
            .user
            .as_ref()
            .map(|u| format!("{u}@"))
            .unwrap_or_default();
        let port_part = if self.port != 22 {
            format!(":{}", self.port)
        } else {
            String::new()
        };

        format!("{}{}{}", user_part, self.host, port_part)
    }
}

/// Manager for SSH profiles
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SshProfileManager {
    profiles: Vec<SshProfile>,
}

impl SshProfileManager {
    /// Create a new empty profile manager
    pub fn new() -> Self {
        Self {
            profiles: Vec::new(),
        }
    }

    /// Add a profile to the manager
    pub fn add(&mut self, profile: SshProfile) {
        // Remove existing profile with same name if exists
        self.profiles.retain(|p| p.name != profile.name);
        self.profiles.push(profile);
    }

    /// Get a profile by name
    pub fn get(&self, name: &str) -> Option<&SshProfile> {
        self.profiles.iter().find(|p| p.name == name)
    }

    /// Remove a profile by name
    /// Returns true if profile was found and removed
    pub fn remove(&mut self, name: &str) -> bool {
        let initial_len = self.profiles.len();
        self.profiles.retain(|p| p.name != name);
        self.profiles.len() < initial_len
    }

    /// List all profiles
    pub fn list(&self) -> &[SshProfile] {
        &self.profiles
    }

    /// Load profiles from SSH config file
    pub fn load_from_ssh_config() -> Self {
        let config_path = match ssh_config_path() {
            Some(path) => path,
            None => {
                warn!("Could not determine SSH config path");
                return Self::new();
            }
        };

        if !config_path.exists() {
            debug!("SSH config file not found at {:?}", config_path);
            return Self::new();
        }

        let content = match fs::read_to_string(&config_path) {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to read SSH config: {}", e);
                return Self::new();
            }
        };

        let parsed = parse_ssh_config(&content);
        let mut manager = Self::new();

        for (hostname, entry) in parsed {
            let profile = SshProfile {
                name: hostname.clone(),
                host: entry
                    .get("hostname")
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| hostname.clone()),
                port: entry
                    .get("port")
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or(22),
                user: entry.get("user").map(|s| s.to_string()),
                identity_file: entry.get("identityfile").map(|s| {
                    let expanded = shellexpand::tilde(s);
                    PathBuf::from(expanded.as_ref())
                }),
                forward_agent: entry
                    .get("forwardagent")
                    .map(|v| v.to_lowercase() == "yes")
                    .unwrap_or(false),
                proxy_jump: entry.get("proxyjump").map(|s| s.to_string()),
                extra_options: Vec::new(),
            };
            manager.add(profile);
        }

        debug!("Loaded {} SSH profiles from config", manager.profiles.len());
        manager
    }
}

/// Get the path to SSH config file
fn ssh_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".ssh").join("config"))
}

/// Parse SSH config file content into a map of host entries
/// Returns: HashMap<hostname, HashMap<key, value>>
fn parse_ssh_config(content: &str) -> HashMap<String, HashMap<String, String>> {
    let mut result = HashMap::new();
    let mut current_host: Option<String> = None;
    let mut current_entry = HashMap::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Split at first whitespace
        let parts: Vec<&str> = line.splitn(2, char::is_whitespace).collect();
        if parts.len() != 2 {
            continue;
        }

        let key = parts[0].to_lowercase();
        let value = parts[1].trim().to_string();

        if key == "host" {
            // Save previous host entry
            if let Some(hostname) = current_host.take() {
                if !current_entry.is_empty() {
                    result.insert(hostname, current_entry);
                    current_entry = HashMap::new();
                }
            }

            // Start new host entry (skip wildcard patterns)
            if !value.contains('*') && !value.contains('?') {
                current_host = Some(value);
            }
        } else if current_host.is_some() {
            // Add option to current host entry
            current_entry.insert(key, value);
        }
    }

    // Save last host entry
    if let Some(hostname) = current_host {
        if !current_entry.is_empty() {
            result.insert(hostname, current_entry);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssh_profile_creation() {
        let profile = SshProfile::new("test".to_string(), "example.com".to_string());
        assert_eq!(profile.name, "test");
        assert_eq!(profile.host, "example.com");
        assert_eq!(profile.port, 22);
        assert!(profile.user.is_none());
    }

    #[test]
    fn test_ssh_command_generation_minimal() {
        let profile = SshProfile::new("test".to_string(), "example.com".to_string());
        let cmd = profile.to_command();
        assert_eq!(cmd, vec!["ssh", "example.com"]);
    }

    #[test]
    fn test_ssh_command_generation_full() {
        let mut profile = SshProfile::new("test".to_string(), "example.com".to_string());
        profile.user = Some("alice".to_string());
        profile.port = 2222;
        profile.identity_file = Some(PathBuf::from("/home/alice/.ssh/id_rsa"));
        profile.forward_agent = true;
        profile.proxy_jump = Some("bastion.example.com".to_string());
        profile.extra_options = vec!["StrictHostKeyChecking=no".to_string()];

        let cmd = profile.to_command();
        assert_eq!(
            cmd,
            vec![
                "ssh",
                "-l",
                "alice",
                "-p",
                "2222",
                "-i",
                "/home/alice/.ssh/id_rsa",
                "-A",
                "-J",
                "bastion.example.com",
                "-o",
                "StrictHostKeyChecking=no",
                "example.com"
            ]
        );
    }

    #[test]
    fn test_connection_string() {
        let mut profile = SshProfile::new("test".to_string(), "example.com".to_string());
        assert_eq!(profile.connection_string(), "example.com");

        profile.user = Some("alice".to_string());
        assert_eq!(profile.connection_string(), "alice@example.com");

        profile.port = 2222;
        assert_eq!(profile.connection_string(), "alice@example.com:2222");
    }

    #[test]
    fn test_ssh_config_parsing() {
        let config = r#"
# Comment line
Host example
    HostName example.com
    User alice
    Port 2222
    IdentityFile ~/.ssh/id_rsa
    ForwardAgent yes

Host bastion
    HostName 10.0.0.1
    User admin
    ProxyJump example

# Another comment
Host *
    ServerAliveInterval 60
"#;

        let parsed = parse_ssh_config(config);

        assert_eq!(parsed.len(), 2); // * is skipped

        let example = parsed.get("example").unwrap();
        assert_eq!(example.get("hostname").unwrap(), "example.com");
        assert_eq!(example.get("user").unwrap(), "alice");
        assert_eq!(example.get("port").unwrap(), "2222");
        assert_eq!(example.get("identityfile").unwrap(), "~/.ssh/id_rsa");
        assert_eq!(example.get("forwardagent").unwrap(), "yes");

        let bastion = parsed.get("bastion").unwrap();
        assert_eq!(bastion.get("hostname").unwrap(), "10.0.0.1");
        assert_eq!(bastion.get("user").unwrap(), "admin");
        assert_eq!(bastion.get("proxyjump").unwrap(), "example");
    }

    #[test]
    fn test_profile_manager_add_get() {
        let mut manager = SshProfileManager::new();
        let profile = SshProfile::new("test".to_string(), "example.com".to_string());

        manager.add(profile.clone());
        assert_eq!(manager.list().len(), 1);

        let retrieved = manager.get("test").unwrap();
        assert_eq!(retrieved.name, "test");
        assert_eq!(retrieved.host, "example.com");
    }

    #[test]
    fn test_profile_manager_remove() {
        let mut manager = SshProfileManager::new();
        let profile = SshProfile::new("test".to_string(), "example.com".to_string());

        manager.add(profile);
        assert_eq!(manager.list().len(), 1);

        let removed = manager.remove("test");
        assert!(removed);
        assert_eq!(manager.list().len(), 0);

        let removed_again = manager.remove("test");
        assert!(!removed_again);
    }

    #[test]
    fn test_profile_manager_add_duplicate() {
        let mut manager = SshProfileManager::new();
        let profile1 = SshProfile::new("test".to_string(), "example.com".to_string());
        let mut profile2 = SshProfile::new("test".to_string(), "other.com".to_string());
        profile2.port = 2222;

        manager.add(profile1);
        manager.add(profile2);

        // Should only have one profile with the name "test"
        assert_eq!(manager.list().len(), 1);

        let profile = manager.get("test").unwrap();
        assert_eq!(profile.host, "other.com");
        assert_eq!(profile.port, 2222);
    }

    #[test]
    fn test_ssh_profile_equality() {
        let profile1 = SshProfile::new("test".to_string(), "example.com".to_string());
        let profile2 = SshProfile::new("test".to_string(), "example.com".to_string());
        let mut profile3 = SshProfile::new("test".to_string(), "example.com".to_string());
        profile3.port = 2222;

        assert_eq!(profile1, profile2);
        assert_ne!(profile1, profile3);
    }
}

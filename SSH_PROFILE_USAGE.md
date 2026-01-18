# SSH Profile Management in AgTerm

AgTerm now supports SSH profile management for easy SSH connections.

## Features

- **Profile Creation**: Create and manage SSH connection profiles
- **SSH Config Parsing**: Automatically load profiles from `~/.ssh/config`
- **Quick Connect**: Open SSH connections in new tabs with a single command
- **Full SSH Options Support**:
  - Custom ports
  - Identity files (private keys)
  - Agent forwarding
  - ProxyJump
  - Custom SSH options

## Configuration

SSH configuration is stored in the main config file under the `[ssh]` section:

```toml
[ssh]
# Automatically detect SSH connection from shell environment
detect_ssh_connection = true

# Load SSH profiles from ~/.ssh/config on startup
load_from_ssh_config = true

# Define custom SSH profiles
[[ssh.profiles]]
name = "production"
host = "prod.example.com"
port = 22
user = "admin"
identity_file = "~/.ssh/id_rsa"
forward_agent = true
extra_options = ["StrictHostKeyChecking=no"]

[[ssh.profiles]]
name = "bastion"
host = "bastion.example.com"
user = "ubuntu"
port = 2222
```

## SSH Profile Structure

Each SSH profile has the following fields:

- **name** (required): Friendly name for the profile
- **host** (required): Remote hostname or IP address
- **port** (optional, default: 22): SSH port
- **user** (optional): Username for connection
- **identity_file** (optional): Path to private key file
- **forward_agent** (optional, default: false): Enable SSH agent forwarding
- **proxy_jump** (optional): ProxyJump host for tunneling
- **extra_options** (optional): Additional SSH options as strings

## Loading from ~/.ssh/config

AgTerm can automatically parse your existing SSH config file. For example:

```ssh-config
Host production
    HostName prod.example.com
    User admin
    Port 22
    IdentityFile ~/.ssh/id_rsa
    ForwardAgent yes

Host bastion
    HostName 10.0.0.1
    User ubuntu
    ProxyJump production
```

These will be automatically loaded and available as SSH profiles.

## Programmatic Usage

### Creating SSH Profiles

```rust
use agterm::ssh::SshProfile;

// Create a basic profile
let profile = SshProfile::new(
    "myserver".to_string(),
    "example.com".to_string()
);

// Create a profile with all options
let mut profile = SshProfile::new(
    "production".to_string(),
    "prod.example.com".to_string()
);
profile.user = Some("admin".to_string());
profile.port = 2222;
profile.identity_file = Some(PathBuf::from("~/.ssh/id_rsa"));
profile.forward_agent = true;
profile.proxy_jump = Some("bastion.example.com".to_string());
profile.extra_options = vec!["StrictHostKeyChecking=no".to_string()];
```

### Managing Profiles

```rust
use agterm::ssh::SshProfileManager;

// Create manager and add profiles
let mut manager = SshProfileManager::new();
manager.add(profile);

// Get a profile by name
if let Some(profile) = manager.get("production") {
    println!("Connecting to: {}", profile.connection_string());
}

// Remove a profile
manager.remove("old-server");

// List all profiles
for profile in manager.list() {
    println!("  - {}: {}", profile.name, profile.connection_string());
}

// Load from SSH config
let manager = SshProfileManager::load_from_ssh_config();
```

### Generating SSH Commands

```rust
// Generate SSH command arguments from profile
let cmd = profile.to_command();
// Returns: ["ssh", "-l", "admin", "-p", "2222", "-i", "~/.ssh/id_rsa", "-A", "prod.example.com"]

// Get connection string for display
let conn_str = profile.connection_string();
// Returns: "admin@prod.example.com:2222"
```

### Loading from SSH Config

```rust
// Load a specific host from ~/.ssh/config
if let Some(profile) = SshProfile::from_ssh_config("production") {
    println!("Loaded profile for {}", profile.name);
}
```

## Opening SSH Tabs (UI Integration)

To open a new SSH tab, send a `Message::NewSshTab` message with an SSH profile:

```rust
// This will:
// 1. Create a new PTY session
// 2. Execute the SSH command
// 3. Set the tab title to "SSH: user@host:port"
Message::NewSshTab(profile)
```

## Example Workflow

1. **Add profiles to config** or let AgTerm load from `~/.ssh/config`
2. **Use command palette** (future feature) to select an SSH profile
3. **New tab opens** with SSH connection already initiated
4. **Start working** on the remote server

## Future Enhancements

Planned features for SSH profile management:

- [ ] UI for managing SSH profiles in settings
- [ ] Command palette integration for quick SSH connections
- [ ] SSH connection status indicator
- [ ] SSH tunnel/port forwarding support
- [ ] SFTP integration for file transfers
- [ ] SSH key generation and management
- [ ] Connection history and recently used profiles
- [ ] Profile groups/categories
- [ ] Import/export profile configurations

## Security Notes

- Private keys are referenced by path, not stored in config
- SSH agent forwarding can be enabled per-profile
- Extra SSH options allow for strict host key checking
- Passwords are not stored (use SSH keys or agent)

## Testing

Run SSH-specific tests:

```bash
cargo test ssh
```

All tests should pass:
- Profile creation and management
- SSH config parsing
- Command generation
- Profile equality and cloning

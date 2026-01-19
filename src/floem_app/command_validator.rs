//! Command validation and risk assessment
//!
//! This module analyzes shell commands to determine their risk level
//! before execution, helping prevent accidental or malicious command execution.

use super::async_bridge::RiskLevel;

/// Command validator for risk assessment
pub struct CommandValidator;

impl CommandValidator {
    /// Assess the risk level of a command
    ///
    /// Analyzes the command string and returns a risk level based on:
    /// - Command name
    /// - Flags and options
    /// - Patterns that indicate destructive operations
    ///
    /// # Examples
    ///
    /// ```
    /// use agterm::floem_app::command_validator::CommandValidator;
    /// use agterm::floem_app::async_bridge::RiskLevel;
    ///
    /// assert_eq!(CommandValidator::assess_risk("ls -la"), RiskLevel::Low);
    /// assert_eq!(CommandValidator::assess_risk("rm -rf /"), RiskLevel::Critical);
    /// ```
    pub fn assess_risk(command: &str) -> RiskLevel {
        let command = command.trim();

        // Empty command is low risk
        if command.is_empty() {
            return RiskLevel::Low;
        }

        // Parse command into tokens (basic shell parsing)
        let tokens = Self::tokenize(command);

        if tokens.is_empty() {
            return RiskLevel::Low;
        }

        // Get the base command (first token, without path)
        let base_command = Self::get_base_command(&tokens[0]);

        // Check for critical patterns first
        if Self::is_critical_command(base_command, &tokens) {
            return RiskLevel::Critical;
        }

        // Check for high-risk commands
        if Self::is_high_risk_command(base_command, &tokens) {
            return RiskLevel::High;
        }

        // Check for medium-risk commands
        if Self::is_medium_risk_command(base_command) {
            return RiskLevel::Medium;
        }

        // Default to low risk
        RiskLevel::Low
    }

    /// Tokenize a command string into separate arguments
    ///
    /// This is a simplified tokenizer that splits on whitespace
    /// while respecting quoted strings and escaped characters.
    fn tokenize(command: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();
        let mut in_quotes = false;
        let mut quote_char = '\0';
        let mut chars = command.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                // Handle quotes
                '"' | '\'' if !in_quotes => {
                    in_quotes = true;
                    quote_char = ch;
                }
                '"' | '\'' if in_quotes && ch == quote_char => {
                    in_quotes = false;
                    quote_char = '\0';
                }
                // Handle escapes
                '\\' if chars.peek().is_some() => {
                    if let Some(next_ch) = chars.next() {
                        current_token.push(next_ch);
                    }
                }
                // Handle whitespace
                ' ' | '\t' if !in_quotes => {
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                }
                // Regular character
                _ => {
                    current_token.push(ch);
                }
            }
        }

        // Add the last token if any
        if !current_token.is_empty() {
            tokens.push(current_token);
        }

        tokens
    }

    /// Extract the base command name from a path
    ///
    /// Examples:
    /// - "/usr/bin/ls" -> "ls"
    /// - "sudo" -> "sudo"
    fn get_base_command(command: &str) -> &str {
        command.rsplit('/').next().unwrap_or(command)
    }

    /// Check if a command is critical risk
    ///
    /// Critical commands can cause:
    /// - Data loss (rm -rf, dd)
    /// - System corruption (mkfs, fdisk)
    /// - Privilege escalation (sudo, su)
    /// - Fork bombs
    fn is_critical_command(base_command: &str, tokens: &[String]) -> bool {
        match base_command {
            // Recursive removal with force flag
            "rm" => Self::has_recursive_force_flags(tokens),

            // Disk operations that can destroy data
            "dd" | "mkfs" | "mkfs.ext4" | "mkfs.ext3" | "mkfs.xfs" | "mkfs.btrfs" => true,

            // Disk partitioning tools
            "fdisk" | "parted" | "gdisk" => true,

            // System privilege commands
            "sudo" | "su" | "doas" => true,

            // Shell commands that can execute arbitrary code
            "eval" | "exec" => true,

            // Shutdown/reboot
            "shutdown" | "reboot" | "halt" | "poweroff" | "init" => true,

            // Kill all processes
            "killall" | "pkill" if Self::has_force_flag(tokens) => true,

            // Format commands
            "format" => true,

            // Wipe commands
            "shred" | "wipe" => true,

            _ => false,
        }
    }

    /// Check if a command is high risk
    ///
    /// High-risk commands can:
    /// - Delete files (rm without -rf)
    /// - Modify system state (chmod, chown)
    /// - Affect processes (kill)
    fn is_high_risk_command(base_command: &str, tokens: &[String]) -> bool {
        match base_command {
            // File removal (without recursive force)
            "rm" => true,

            // Permission changes
            "chmod" | "chown" | "chgrp" => true,

            // Process termination
            "kill" | "killall" | "pkill" => true,

            // Package management (can modify system)
            "apt" | "apt-get" | "yum" | "dnf" | "pacman" | "brew" => {
                Self::is_package_modify_operation(tokens)
            }

            // Linking operations (can overwrite files)
            "ln" if Self::has_flag(tokens, "-f") || Self::has_flag(tokens, "--force") => true,

            // Truncate/overwrite files
            "truncate" => true,

            // Archive extraction (can overwrite files)
            "tar" if Self::has_extract_flag(tokens) => true,

            // Docker operations
            "docker" if Self::is_docker_risky_operation(tokens) => true,

            _ => false,
        }
    }

    /// Check if a command is medium risk
    ///
    /// Medium-risk commands can:
    /// - Create files/directories
    /// - Move/copy files
    /// - Modify files
    fn is_medium_risk_command(base_command: &str) -> bool {
        matches!(
            base_command,
            // File operations
            "cp" | "mv" | "mkdir" | "rmdir" | "touch" |
            // Text editing
            "vim" | "vi" | "nano" | "emacs" | "ed" | "sed" |
            // File writing
            "tee" |
            // Archive operations
            "tar" | "zip" | "unzip" | "gzip" | "gunzip" | "bzip2" | "bunzip2" |
            // Download operations
            "wget" | "curl" |
            // Git operations that modify state
            "git" |
            // Make/build operations
            "make" | "cmake" | "cargo" | "npm" | "yarn" | "pip" |
            // Link operations (without force)
            "ln" |
            // Scripting
            "sh" | "bash" | "zsh" | "fish" | "python" | "ruby" | "perl" | "node"
        )
    }

    /// Check if tokens contain recursive and force flags for rm
    fn has_recursive_force_flags(tokens: &[String]) -> bool {
        let has_recursive = tokens.iter().any(|t| {
            t == "-r" || t == "-R" || t == "--recursive" || t.contains('r') && t.starts_with('-') && !t.starts_with("--")
        });

        let has_force = tokens.iter().any(|t| {
            t == "-f" || t == "--force" || t.contains('f') && t.starts_with('-') && !t.starts_with("--")
        });

        has_recursive && has_force
    }

    /// Check if tokens contain a force flag
    fn has_force_flag(tokens: &[String]) -> bool {
        tokens.iter().any(|t| {
            t == "-f" || t == "--force" || t == "-9" || t == "-KILL"
        })
    }

    /// Check if tokens contain a specific flag
    fn has_flag(tokens: &[String], flag: &str) -> bool {
        tokens.iter().any(|t| t == flag)
    }

    /// Check if tar command has extract flags
    fn has_extract_flag(tokens: &[String]) -> bool {
        tokens.iter().any(|t| {
            (t.starts_with('-') && t.contains('x')) || t == "--extract"
        })
    }

    /// Check if package operation modifies the system
    fn is_package_modify_operation(tokens: &[String]) -> bool {
        tokens.iter().any(|t| {
            matches!(
                t.as_str(),
                "install" | "remove" | "purge" | "update" | "upgrade" | "dist-upgrade"
            )
        })
    }

    /// Check if Docker operation is risky
    fn is_docker_risky_operation(tokens: &[String]) -> bool {
        tokens.iter().any(|t| {
            matches!(
                t.as_str(),
                "rm" | "rmi" | "prune" | "kill" | "stop" | "system"
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low_risk_commands() {
        assert_eq!(CommandValidator::assess_risk("ls"), RiskLevel::Low);
        assert_eq!(CommandValidator::assess_risk("ls -la"), RiskLevel::Low);
        assert_eq!(CommandValidator::assess_risk("cat file.txt"), RiskLevel::Low);
        assert_eq!(CommandValidator::assess_risk("pwd"), RiskLevel::Low);
        assert_eq!(CommandValidator::assess_risk("echo hello"), RiskLevel::Low);
        assert_eq!(CommandValidator::assess_risk("date"), RiskLevel::Low);
        assert_eq!(CommandValidator::assess_risk("whoami"), RiskLevel::Low);
        assert_eq!(CommandValidator::assess_risk("which ls"), RiskLevel::Low);
        assert_eq!(CommandValidator::assess_risk("grep pattern file"), RiskLevel::Low);
        assert_eq!(CommandValidator::assess_risk("find . -name '*.rs'"), RiskLevel::Low);
    }

    #[test]
    fn test_medium_risk_commands() {
        assert_eq!(CommandValidator::assess_risk("cp file1 file2"), RiskLevel::Medium);
        assert_eq!(CommandValidator::assess_risk("mv file1 file2"), RiskLevel::Medium);
        assert_eq!(CommandValidator::assess_risk("mkdir newdir"), RiskLevel::Medium);
        assert_eq!(CommandValidator::assess_risk("touch newfile"), RiskLevel::Medium);
        assert_eq!(CommandValidator::assess_risk("vim file.txt"), RiskLevel::Medium);
        assert_eq!(CommandValidator::assess_risk("git commit -m 'message'"), RiskLevel::Medium);
    }

    #[test]
    fn test_high_risk_commands() {
        assert_eq!(CommandValidator::assess_risk("rm file.txt"), RiskLevel::High);
        assert_eq!(CommandValidator::assess_risk("chmod 777 file"), RiskLevel::High);
        assert_eq!(CommandValidator::assess_risk("chown user file"), RiskLevel::High);
        assert_eq!(CommandValidator::assess_risk("kill 1234"), RiskLevel::High);
        assert_eq!(CommandValidator::assess_risk("apt install package"), RiskLevel::High);
    }

    #[test]
    fn test_critical_risk_commands() {
        assert_eq!(CommandValidator::assess_risk("rm -rf /"), RiskLevel::Critical);
        assert_eq!(CommandValidator::assess_risk("rm -rf *"), RiskLevel::Critical);
        assert_eq!(CommandValidator::assess_risk("sudo rm file"), RiskLevel::Critical);
        assert_eq!(CommandValidator::assess_risk("dd if=/dev/zero of=/dev/sda"), RiskLevel::Critical);
        assert_eq!(CommandValidator::assess_risk("mkfs.ext4 /dev/sda"), RiskLevel::Critical);
        assert_eq!(CommandValidator::assess_risk("shutdown now"), RiskLevel::Critical);
    }

    #[test]
    fn test_rm_variants() {
        // Safe rm
        assert_eq!(CommandValidator::assess_risk("rm file.txt"), RiskLevel::High);
        assert_eq!(CommandValidator::assess_risk("rm -i file.txt"), RiskLevel::High);

        // Dangerous rm
        assert_eq!(CommandValidator::assess_risk("rm -rf dir"), RiskLevel::Critical);
        assert_eq!(CommandValidator::assess_risk("rm -fr dir"), RiskLevel::Critical);
        assert_eq!(CommandValidator::assess_risk("rm -f -r dir"), RiskLevel::Critical);
    }

    #[test]
    fn test_tokenize() {
        let tokens = CommandValidator::tokenize("ls -la /tmp");
        assert_eq!(tokens, vec!["ls", "-la", "/tmp"]);

        let tokens = CommandValidator::tokenize("echo 'hello world'");
        assert_eq!(tokens, vec!["echo", "hello world"]);

        let tokens = CommandValidator::tokenize("echo \"hello world\"");
        assert_eq!(tokens, vec!["echo", "hello world"]);

        let tokens = CommandValidator::tokenize("rm -rf /tmp/test\\ dir");
        assert_eq!(tokens, vec!["rm", "-rf", "/tmp/test dir"]);
    }

    #[test]
    fn test_get_base_command() {
        assert_eq!(CommandValidator::get_base_command("ls"), "ls");
        assert_eq!(CommandValidator::get_base_command("/usr/bin/ls"), "ls");
        assert_eq!(CommandValidator::get_base_command("/bin/bash"), "bash");
    }

    #[test]
    fn test_empty_command() {
        assert_eq!(CommandValidator::assess_risk(""), RiskLevel::Low);
        assert_eq!(CommandValidator::assess_risk("   "), RiskLevel::Low);
    }

    #[test]
    fn test_package_managers() {
        assert_eq!(CommandValidator::assess_risk("apt list"), RiskLevel::Low);
        assert_eq!(CommandValidator::assess_risk("apt install vim"), RiskLevel::High);
        assert_eq!(CommandValidator::assess_risk("brew install git"), RiskLevel::High);
    }

    #[test]
    fn test_docker_commands() {
        assert_eq!(CommandValidator::assess_risk("docker ps"), RiskLevel::Low);
        assert_eq!(CommandValidator::assess_risk("docker images"), RiskLevel::Low);
        assert_eq!(CommandValidator::assess_risk("docker rm container"), RiskLevel::High);
        assert_eq!(CommandValidator::assess_risk("docker rmi image"), RiskLevel::High);
    }
}

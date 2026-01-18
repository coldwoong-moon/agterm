use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Supported shell types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Nushell,
    Unknown,
}

impl ShellType {
    /// Detect shell type from shell name
    pub fn from_name(name: &str) -> Self {
        let name_lower = name.to_lowercase();
        if name_lower.contains("bash") {
            ShellType::Bash
        } else if name_lower.contains("zsh") {
            ShellType::Zsh
        } else if name_lower.contains("fish") {
            ShellType::Fish
        } else if name_lower.contains("pwsh") || name_lower.contains("powershell") {
            ShellType::PowerShell
        } else if name_lower.contains("nu") {
            ShellType::Nushell
        } else {
            ShellType::Unknown
        }
    }
}

/// Shell integration features
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Feature {
    CwdTracking,
    CommandHistory,
    PromptMarks,
    ExitCodeReporting,
    CommandTiming,
    CompletionIntegration,
    TitleIntegration,
    GitIntegration,
    DirectoryJump,
}

/// Main shell integration state
#[derive(Debug)]
pub struct ShellIntegration {
    pub shell_type: ShellType,
    pub enabled_features: HashSet<Feature>,
    pub prompt_command: Option<String>,
    pub cwd: PathBuf,
    pub last_command: Option<String>,
    pub last_exit_code: Option<i32>,
    pub command_start_time: Option<Instant>,
    pub command_tracker: CommandTracker,
    pub directory_history: DirectoryHistory,
    pub prompt_info: PromptInfo,
}

impl ShellIntegration {
    pub fn new(shell_type: ShellType) -> Self {
        let mut enabled_features = HashSet::new();
        enabled_features.insert(Feature::CwdTracking);
        enabled_features.insert(Feature::CommandHistory);
        enabled_features.insert(Feature::PromptMarks);
        enabled_features.insert(Feature::ExitCodeReporting);
        enabled_features.insert(Feature::CommandTiming);

        Self {
            shell_type,
            enabled_features,
            prompt_command: None,
            cwd: PathBuf::from("/"),
            last_command: None,
            last_exit_code: None,
            command_start_time: None,
            command_tracker: CommandTracker::new(),
            directory_history: DirectoryHistory::new(),
            prompt_info: PromptInfo::default(),
        }
    }

    /// Handle a shell event
    pub fn handle_event(&mut self, event: ShellEvent) {
        match event {
            ShellEvent::CwdChanged(path) => {
                self.cwd = path.clone();
                self.prompt_info.cwd = path.clone();
                self.prompt_info.cwd_short = abbreviate_path(&path);
                self.directory_history.push(path);
            }
            ShellEvent::CommandStarted(cmd) => {
                self.last_command = Some(cmd.clone());
                self.command_start_time = Some(Instant::now());
                self.command_tracker.start_command(&cmd);
            }
            ShellEvent::CommandFinished { exit_code, duration } => {
                self.last_exit_code = Some(exit_code);
                self.prompt_info.last_exit_code = Some(exit_code);
                self.prompt_info.execution_time = Some(duration);
                self.command_tracker.end_command(exit_code);
                self.command_start_time = None;
            }
            ShellEvent::PromptStart => {
                // Reset prompt info for new prompt
            }
            ShellEvent::PromptEnd => {
                // Prompt rendering complete
            }
            ShellEvent::OutputStart => {
                // Command output started
            }
            ShellEvent::OutputEnd => {
                // Command output ended
            }
            ShellEvent::GitStatusChanged(status) => {
                self.prompt_info.git_branch = Some(status.branch.clone());
                self.prompt_info.git_status = Some(status);
            }
        }
    }

    /// Parse OSC (Operating System Command) sequences
    pub fn parse_osc(&mut self, command: u32, data: &str) -> Option<ShellEvent> {
        match command {
            // OSC 7: Current working directory
            7 => {
                if let Some(path) = parse_osc7_cwd(data) {
                    return Some(ShellEvent::CwdChanged(path));
                }
            }
            // OSC 133: Shell integration marks
            133 => {
                return parse_osc133_mark(data);
            }
            // OSC 1337: iTerm2-style integration
            1337 => {
                return parse_osc1337(data);
            }
            _ => {}
        }
        None
    }

    /// Enable a feature
    pub fn enable_feature(&mut self, feature: Feature) {
        self.enabled_features.insert(feature);
    }

    /// Disable a feature
    pub fn disable_feature(&mut self, feature: Feature) {
        self.enabled_features.remove(&feature);
    }

    /// Check if a feature is enabled
    pub fn is_feature_enabled(&self, feature: Feature) -> bool {
        self.enabled_features.contains(&feature)
    }

    /// Get integration script for current shell
    pub fn get_integration_script(&self) -> String {
        integration_script(self.shell_type)
    }
}

/// Shell events
#[derive(Debug, Clone)]
pub enum ShellEvent {
    CwdChanged(PathBuf),
    CommandStarted(String),
    CommandFinished { exit_code: i32, duration: Duration },
    PromptStart,
    PromptEnd,
    OutputStart,
    OutputEnd,
    GitStatusChanged(GitStatus),
}

/// Prompt information
#[derive(Debug, Clone, Default)]
pub struct PromptInfo {
    pub user: Option<String>,
    pub hostname: Option<String>,
    pub cwd: PathBuf,
    pub cwd_short: String,
    pub git_branch: Option<String>,
    pub git_status: Option<GitStatus>,
    pub virtualenv: Option<String>,
    pub last_exit_code: Option<i32>,
    pub execution_time: Option<Duration>,
}

/// Git status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub branch: String,
    pub ahead: u32,
    pub behind: u32,
    pub staged: u32,
    pub unstaged: u32,
    pub untracked: u32,
    pub conflicts: u32,
    pub is_clean: bool,
    pub is_detached: bool,
    pub stash_count: u32,
}

impl GitStatus {
    pub fn new(branch: String) -> Self {
        Self {
            branch,
            ahead: 0,
            behind: 0,
            staged: 0,
            unstaged: 0,
            untracked: 0,
            conflicts: 0,
            is_clean: true,
            is_detached: false,
            stash_count: 0,
        }
    }

    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        self.staged > 0 || self.unstaged > 0 || self.untracked > 0
    }

    /// Check if there are unpushed commits
    pub fn has_unpushed(&self) -> bool {
        self.ahead > 0
    }

    /// Check if there are unpulled commits
    pub fn has_unpulled(&self) -> bool {
        self.behind > 0
    }
}

/// Command execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExecution {
    pub command: String,
    pub start_time: DateTime<Utc>,
    pub duration: Duration,
    pub exit_code: i32,
    pub cwd: PathBuf,
}

/// Command tracker
#[derive(Debug, Serialize, Deserialize)]
pub struct CommandTracker {
    pub history: Vec<CommandExecution>,
    #[serde(skip)]
    current_command: Option<String>,
    #[serde(skip)]
    current_start: Option<Instant>,
    #[serde(skip)]
    current_cwd: PathBuf,
    max_history: usize,
}

impl CommandTracker {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            current_command: None,
            current_start: None,
            current_cwd: PathBuf::from("/"),
            max_history: 1000,
        }
    }

    pub fn with_max_history(max_history: usize) -> Self {
        Self {
            history: Vec::new(),
            current_command: None,
            current_start: None,
            current_cwd: PathBuf::from("/"),
            max_history,
        }
    }

    /// Start tracking a command
    pub fn start_command(&mut self, cmd: &str) {
        self.current_command = Some(cmd.to_string());
        self.current_start = Some(Instant::now());
    }

    /// End tracking a command
    pub fn end_command(&mut self, exit_code: i32) {
        if let (Some(cmd), Some(start)) = (self.current_command.take(), self.current_start.take())
        {
            let duration = start.elapsed();
            let execution = CommandExecution {
                command: cmd,
                start_time: Utc::now(),
                duration,
                exit_code,
                cwd: self.current_cwd.clone(),
            };

            self.history.push(execution);

            // Limit history size
            if self.history.len() > self.max_history {
                self.history.remove(0);
            }
        }
    }

    /// Get current command being executed
    pub fn current_command(&self) -> Option<&str> {
        self.current_command.as_deref()
    }

    /// Get duration of last command
    pub fn last_duration(&self) -> Option<Duration> {
        self.history.last().map(|e| e.duration)
    }

    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.history.is_empty() {
            return 0.0;
        }

        let successful = self.history.iter().filter(|e| e.exit_code == 0).count();
        successful as f64 / self.history.len() as f64
    }

    /// Update current working directory
    pub fn set_cwd(&mut self, cwd: PathBuf) {
        self.current_cwd = cwd;
    }

    /// Get commands by exit code
    pub fn commands_by_exit_code(&self, exit_code: i32) -> Vec<&CommandExecution> {
        self.history
            .iter()
            .filter(|e| e.exit_code == exit_code)
            .collect()
    }

    /// Get average execution time
    pub fn average_duration(&self) -> Option<Duration> {
        if self.history.is_empty() {
            return None;
        }

        let total: Duration = self.history.iter().map(|e| e.duration).sum();
        Some(total / self.history.len() as u32)
    }
}

impl Default for CommandTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Directory history and navigation
#[derive(Debug, Serialize, Deserialize)]
pub struct DirectoryHistory {
    history: VecDeque<PathBuf>,
    frequency: HashMap<PathBuf, usize>,
    current_index: usize,
    max_history: usize,
}

impl DirectoryHistory {
    pub fn new() -> Self {
        Self {
            history: VecDeque::new(),
            frequency: HashMap::new(),
            current_index: 0,
            max_history: 100,
        }
    }

    pub fn with_max_history(max_history: usize) -> Self {
        Self {
            history: VecDeque::new(),
            frequency: HashMap::new(),
            current_index: 0,
            max_history,
        }
    }

    /// Record a directory visit
    pub fn push(&mut self, path: PathBuf) {
        // Update frequency
        *self.frequency.entry(path.clone()).or_insert(0) += 1;

        // Add to history
        self.history.push_back(path);
        self.current_index = self.history.len().saturating_sub(1);

        // Limit history size
        if self.history.len() > self.max_history {
            if let Some(removed) = self.history.pop_front() {
                if let Some(count) = self.frequency.get_mut(&removed) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        self.frequency.remove(&removed);
                    }
                }
            }
            self.current_index = self.current_index.saturating_sub(1);
        }
    }

    /// Get recent directories
    pub fn recent(&self, limit: usize) -> Vec<&PathBuf> {
        self.history
            .iter()
            .rev()
            .take(limit)
            .collect()
    }

    /// Get most frequent directories
    pub fn frequent(&self, limit: usize) -> Vec<&PathBuf> {
        let mut entries: Vec<_> = self.frequency.iter().collect();
        entries.sort_by(|a, b| b.1.cmp(a.1));
        entries
            .into_iter()
            .take(limit)
            .map(|(path, _)| path)
            .collect()
    }

    /// Fuzzy find directory
    pub fn jump(&self, query: &str) -> Option<&PathBuf> {
        let query_lower = query.to_lowercase();

        // First try exact match
        for path in self.history.iter().rev() {
            if let Some(name) = path.file_name() {
                if name.to_string_lossy().to_lowercase() == query_lower {
                    return Some(path);
                }
            }
        }

        // Then try substring match
        for path in self.history.iter().rev() {
            if path.to_string_lossy().to_lowercase().contains(&query_lower) {
                return Some(path);
            }
        }

        None
    }

    /// Navigate back in history
    pub fn back(&mut self) -> Option<&PathBuf> {
        if self.current_index > 0 {
            self.current_index -= 1;
            self.history.get(self.current_index)
        } else {
            None
        }
    }

    /// Navigate forward in history
    pub fn forward(&mut self) -> Option<&PathBuf> {
        if self.current_index < self.history.len().saturating_sub(1) {
            self.current_index += 1;
            self.history.get(self.current_index)
        } else {
            None
        }
    }

    /// Get current directory
    pub fn current(&self) -> Option<&PathBuf> {
        self.history.back()
    }

    /// Clear history
    pub fn clear(&mut self) {
        self.history.clear();
        self.frequency.clear();
        self.current_index = 0;
    }
}

impl Default for DirectoryHistory {
    fn default() -> Self {
        Self::new()
    }
}

// OSC parsing functions

/// Parse OSC 7 (current working directory)
fn parse_osc7_cwd(data: &str) -> Option<PathBuf> {
    // Format: file://hostname/path
    if let Some(path_start) = data.find("file://") {
        let path_data = &data[path_start + 7..];
        if let Some(slash_pos) = path_data.find('/') {
            let path = &path_data[slash_pos..];
            // URL decode
            if let Ok(decoded) = urlencoding::decode(path) {
                return Some(PathBuf::from(decoded.to_string()));
            }
        }
    }
    None
}

/// Parse OSC 133 (shell integration marks)
fn parse_osc133_mark(data: &str) -> Option<ShellEvent> {
    // Format: 133;X where X is A (prompt start), B (prompt end), C (output start), D (output end)
    let parts: Vec<&str> = data.split(';').collect();
    if parts.len() >= 2 && parts[0] == "133" {
        match parts[1] {
            "A" => return Some(ShellEvent::PromptStart),
            "B" => return Some(ShellEvent::PromptEnd),
            "C" => return Some(ShellEvent::OutputStart),
            "D" => {
                // OSC 133;D;exit_code
                if parts.len() >= 3 {
                    if let Ok(exit_code) = parts[2].parse::<i32>() {
                        return Some(ShellEvent::CommandFinished {
                            exit_code,
                            duration: Duration::from_secs(0),
                        });
                    }
                }
                return Some(ShellEvent::OutputEnd);
            }
            _ => {}
        }
    }
    None
}

/// Parse OSC 1337 (iTerm2-style integration)
fn parse_osc1337(data: &str) -> Option<ShellEvent> {
    // Format: 1337;key=value
    let parts: Vec<&str> = data.split(';').collect();
    if parts.len() >= 2 && parts[0] == "1337" {
        for part in &parts[1..] {
            if let Some((key, value)) = part.split_once('=') {
                match key {
                    "CurrentDir" => {
                        if let Ok(decoded) = urlencoding::decode(value) {
                            return Some(ShellEvent::CwdChanged(PathBuf::from(decoded.to_string())));
                        }
                    }
                    "RemoteHost" => {
                        // Could store remote host info
                    }
                    _ => {}
                }
            }
        }
    }
    None
}

// Integration script generators

/// Generate bash integration script
pub fn generate_bash_integration() -> String {
    r#"# AgTerm Shell Integration for Bash

# Report current directory
__agterm_report_cwd() {
    printf '\e]7;file://%s%s\a' "$HOSTNAME" "$PWD"
}

# Command start marker
__agterm_preexec() {
    printf '\e]133;C\a'
}

# Command end marker
__agterm_precmd() {
    local exit_code=$?
    printf '\e]133;D;%s\a' "$exit_code"
    __agterm_report_cwd
    printf '\e]133;A\a'
}

# Prompt marker
__agterm_prompt_end() {
    printf '\e]133;B\a'
}

# Install hooks
if [[ -z "${PROMPT_COMMAND}" ]]; then
    PROMPT_COMMAND="__agterm_precmd"
else
    PROMPT_COMMAND="${PROMPT_COMMAND}; __agterm_precmd"
fi

# Add prompt marker to PS1
if [[ "$PS1" != *'__agterm_prompt_end'* ]]; then
    PS1="${PS1}\$(__agterm_prompt_end)"
fi

# Initial directory report
__agterm_report_cwd

# Directory jump function
j() {
    if [ -z "$1" ]; then
        cd ~
    else
        # This would integrate with AgTerm's directory history
        printf '\e]1337;Jump=%s\a' "$1"
    fi
}
"#.to_string()
}

/// Generate zsh integration script
pub fn generate_zsh_integration() -> String {
    r#"# AgTerm Shell Integration for Zsh

# Report current directory
__agterm_report_cwd() {
    printf '\e]7;file://%s%s\a' "$HOST" "$PWD"
}

# Command start marker
__agterm_preexec() {
    printf '\e]133;C\a'
}

# Command end marker
__agterm_precmd() {
    local exit_code=$?
    printf '\e]133;D;%s\a' "$exit_code"
    __agterm_report_cwd
    printf '\e]133;A\a'
}

# Prompt marker
__agterm_prompt_end() {
    printf '\e]133;B\a'
}

# Install hooks
autoload -Uz add-zsh-hook
add-zsh-hook precmd __agterm_precmd
add-zsh-hook preexec __agterm_preexec

# Add prompt marker to PS1
if [[ "$PS1" != *'__agterm_prompt_end'* ]]; then
    PS1="${PS1}\$(__agterm_prompt_end)"
fi

# Initial directory report
__agterm_report_cwd

# Directory jump function
j() {
    if [ -z "$1" ]; then
        cd ~
    else
        printf '\e]1337;Jump=%s\a' "$1"
    fi
}
"#.to_string()
}

/// Generate fish integration script
pub fn generate_fish_integration() -> String {
    r#"# AgTerm Shell Integration for Fish

# Report current directory
function __agterm_report_cwd --on-variable PWD
    printf '\e]7;file://%s%s\a' (hostname) (pwd)
end

# Command start marker
function __agterm_preexec --on-event fish_preexec
    printf '\e]133;C\a'
end

# Command end marker
function __agterm_precmd --on-event fish_prompt
    set -l exit_code $status
    printf '\e]133;D;%s\a' $exit_code
    __agterm_report_cwd
    printf '\e]133;A\a'
end

# Prompt marker
function __agterm_prompt_end
    printf '\e]133;B\a'
end

# Add prompt marker to prompt
if not contains __agterm_prompt_end $fish_prompt
    function fish_prompt
        printf '%s%s' (fish_prompt) (__agterm_prompt_end)
    end
end

# Initial directory report
__agterm_report_cwd

# Directory jump function
function j
    if test (count $argv) -eq 0
        cd ~
    else
        printf '\e]1337;Jump=%s\a' $argv[1]
    end
end
"#.to_string()
}

/// Get integration script for a specific shell
pub fn integration_script(shell: ShellType) -> String {
    match shell {
        ShellType::Bash => generate_bash_integration(),
        ShellType::Zsh => generate_zsh_integration(),
        ShellType::Fish => generate_fish_integration(),
        ShellType::PowerShell => {
            "# PowerShell integration not yet implemented".to_string()
        }
        ShellType::Nushell => {
            "# Nushell integration not yet implemented".to_string()
        }
        ShellType::Unknown => {
            "# Unknown shell type - integration not available".to_string()
        }
    }
}

// Helper functions

/// Abbreviate a path for display
fn abbreviate_path(path: &PathBuf) -> String {
    let path_str = path.to_string_lossy();

    // Replace home directory with ~
    if let Some(home) = std::env::var_os("HOME") {
        let home_str = PathBuf::from(home).to_string_lossy().to_string();
        if path_str.starts_with(&home_str) {
            return path_str.replacen(&home_str, "~", 1);
        }
    }

    path_str.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_type_detection() {
        assert_eq!(ShellType::from_name("bash"), ShellType::Bash);
        assert_eq!(ShellType::from_name("/bin/bash"), ShellType::Bash);
        assert_eq!(ShellType::from_name("zsh"), ShellType::Zsh);
        assert_eq!(ShellType::from_name("/usr/bin/zsh"), ShellType::Zsh);
        assert_eq!(ShellType::from_name("fish"), ShellType::Fish);
        assert_eq!(ShellType::from_name("pwsh"), ShellType::PowerShell);
        assert_eq!(ShellType::from_name("nu"), ShellType::Nushell);
        assert_eq!(ShellType::from_name("unknown"), ShellType::Unknown);
    }

    #[test]
    fn test_shell_integration_creation() {
        let integration = ShellIntegration::new(ShellType::Bash);
        assert_eq!(integration.shell_type, ShellType::Bash);
        assert!(integration.is_feature_enabled(Feature::CwdTracking));
        assert!(integration.is_feature_enabled(Feature::CommandHistory));
        assert!(integration.is_feature_enabled(Feature::PromptMarks));
    }

    #[test]
    fn test_feature_management() {
        let mut integration = ShellIntegration::new(ShellType::Bash);

        integration.enable_feature(Feature::GitIntegration);
        assert!(integration.is_feature_enabled(Feature::GitIntegration));

        integration.disable_feature(Feature::GitIntegration);
        assert!(!integration.is_feature_enabled(Feature::GitIntegration));
    }

    #[test]
    fn test_osc7_parsing() {
        let path = parse_osc7_cwd("file://hostname/home/user/project");
        assert_eq!(path, Some(PathBuf::from("/home/user/project")));

        let path = parse_osc7_cwd("file://hostname/Users/test/Documents");
        assert_eq!(path, Some(PathBuf::from("/Users/test/Documents")));
    }

    #[test]
    fn test_osc7_url_encoding() {
        let path = parse_osc7_cwd("file://hostname/home/user/my%20folder");
        assert_eq!(path, Some(PathBuf::from("/home/user/my folder")));
    }

    #[test]
    fn test_osc133_prompt_start() {
        let event = parse_osc133_mark("133;A");
        assert!(matches!(event, Some(ShellEvent::PromptStart)));
    }

    #[test]
    fn test_osc133_prompt_end() {
        let event = parse_osc133_mark("133;B");
        assert!(matches!(event, Some(ShellEvent::PromptEnd)));
    }

    #[test]
    fn test_osc133_output_start() {
        let event = parse_osc133_mark("133;C");
        assert!(matches!(event, Some(ShellEvent::OutputStart)));
    }

    #[test]
    fn test_osc133_output_end() {
        let event = parse_osc133_mark("133;D");
        assert!(matches!(event, Some(ShellEvent::OutputEnd)));
    }

    #[test]
    fn test_osc133_exit_code() {
        let event = parse_osc133_mark("133;D;0");
        assert!(matches!(
            event,
            Some(ShellEvent::CommandFinished { exit_code: 0, .. })
        ));

        let event = parse_osc133_mark("133;D;127");
        assert!(matches!(
            event,
            Some(ShellEvent::CommandFinished { exit_code: 127, .. })
        ));
    }

    #[test]
    fn test_osc1337_current_dir() {
        let event = parse_osc1337("1337;CurrentDir=/home/user");
        assert!(matches!(event, Some(ShellEvent::CwdChanged(_))));
    }

    #[test]
    fn test_command_tracker_basic() {
        let mut tracker = CommandTracker::new();

        tracker.start_command("ls -la");
        assert_eq!(tracker.current_command(), Some("ls -la"));

        std::thread::sleep(Duration::from_millis(10));
        tracker.end_command(0);

        assert_eq!(tracker.current_command(), None);
        assert_eq!(tracker.history.len(), 1);
        assert_eq!(tracker.history[0].exit_code, 0);
    }

    #[test]
    fn test_command_tracker_success_rate() {
        let mut tracker = CommandTracker::new();

        tracker.start_command("cmd1");
        tracker.end_command(0);

        tracker.start_command("cmd2");
        tracker.end_command(1);

        tracker.start_command("cmd3");
        tracker.end_command(0);

        assert_eq!(tracker.success_rate(), 2.0 / 3.0);
    }

    #[test]
    fn test_command_tracker_max_history() {
        let mut tracker = CommandTracker::with_max_history(3);

        for i in 0..5 {
            tracker.start_command(&format!("cmd{}", i));
            tracker.end_command(0);
        }

        assert_eq!(tracker.history.len(), 3);
        assert_eq!(tracker.history[0].command, "cmd2");
    }

    #[test]
    fn test_directory_history_push() {
        let mut history = DirectoryHistory::new();

        history.push(PathBuf::from("/home/user"));
        history.push(PathBuf::from("/var/log"));

        assert_eq!(history.history.len(), 2);
        assert_eq!(history.current(), Some(&PathBuf::from("/var/log")));
    }

    #[test]
    fn test_directory_history_recent() {
        let mut history = DirectoryHistory::new();

        history.push(PathBuf::from("/a"));
        history.push(PathBuf::from("/b"));
        history.push(PathBuf::from("/c"));

        let recent = history.recent(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0], &PathBuf::from("/c"));
        assert_eq!(recent[1], &PathBuf::from("/b"));
    }

    #[test]
    fn test_directory_history_frequent() {
        let mut history = DirectoryHistory::new();

        history.push(PathBuf::from("/a"));
        history.push(PathBuf::from("/b"));
        history.push(PathBuf::from("/a"));
        history.push(PathBuf::from("/c"));
        history.push(PathBuf::from("/a"));

        let frequent = history.frequent(1);
        assert_eq!(frequent.len(), 1);
        assert_eq!(frequent[0], &PathBuf::from("/a"));
    }

    #[test]
    fn test_directory_history_jump() {
        let mut history = DirectoryHistory::new();

        history.push(PathBuf::from("/home/user/project"));
        history.push(PathBuf::from("/var/log"));

        let result = history.jump("project");
        assert_eq!(result, Some(&PathBuf::from("/home/user/project")));

        let result = history.jump("log");
        assert_eq!(result, Some(&PathBuf::from("/var/log")));

        let result = history.jump("nonexistent");
        assert_eq!(result, None);
    }

    #[test]
    fn test_directory_history_navigation() {
        let mut history = DirectoryHistory::new();

        history.push(PathBuf::from("/a"));
        history.push(PathBuf::from("/b"));
        history.push(PathBuf::from("/c"));

        let back = history.back();
        assert_eq!(back, Some(&PathBuf::from("/b")));

        let back = history.back();
        assert_eq!(back, Some(&PathBuf::from("/a")));

        let forward = history.forward();
        assert_eq!(forward, Some(&PathBuf::from("/b")));
    }

    #[test]
    fn test_git_status() {
        let mut status = GitStatus::new("main".to_string());
        assert!(status.is_clean);
        assert!(!status.has_changes());

        status.staged = 2;
        status.unstaged = 1;
        status.is_clean = false;
        assert!(status.has_changes());
        assert!(!status.is_clean);

        status.ahead = 3;
        assert!(status.has_unpushed());

        status.behind = 1;
        assert!(status.has_unpulled());
    }

    #[test]
    fn test_shell_event_handling() {
        let mut integration = ShellIntegration::new(ShellType::Bash);

        let event = ShellEvent::CwdChanged(PathBuf::from("/home/user"));
        integration.handle_event(event);
        assert_eq!(integration.cwd, PathBuf::from("/home/user"));

        let event = ShellEvent::CommandStarted("ls".to_string());
        integration.handle_event(event);
        assert_eq!(integration.last_command, Some("ls".to_string()));

        let event = ShellEvent::CommandFinished {
            exit_code: 0,
            duration: Duration::from_secs(1),
        };
        integration.handle_event(event);
        assert_eq!(integration.last_exit_code, Some(0));
    }

    #[test]
    fn test_integration_scripts_generated() {
        let bash_script = generate_bash_integration();
        assert!(bash_script.contains("AgTerm"));
        assert!(bash_script.contains("__agterm_report_cwd"));

        let zsh_script = generate_zsh_integration();
        assert!(zsh_script.contains("AgTerm"));
        assert!(zsh_script.contains("add-zsh-hook"));

        let fish_script = generate_fish_integration();
        assert!(fish_script.contains("AgTerm"));
        assert!(fish_script.contains("fish_prompt"));
    }

    #[test]
    fn test_abbreviate_path() {
        let path = PathBuf::from("/home/user/very/long/path/to/project");
        let abbrev = abbreviate_path(&path);
        assert!(abbrev.contains("project"));
    }

    #[test]
    fn test_command_tracker_average_duration() {
        let mut tracker = CommandTracker::new();

        tracker.start_command("cmd1");
        std::thread::sleep(Duration::from_millis(10));
        tracker.end_command(0);

        tracker.start_command("cmd2");
        std::thread::sleep(Duration::from_millis(10));
        tracker.end_command(0);

        let avg = tracker.average_duration();
        assert!(avg.is_some());
        assert!(avg.unwrap() >= Duration::from_millis(10));
    }

    #[test]
    fn test_commands_by_exit_code() {
        let mut tracker = CommandTracker::new();

        tracker.start_command("success");
        tracker.end_command(0);

        tracker.start_command("failure");
        tracker.end_command(1);

        tracker.start_command("success2");
        tracker.end_command(0);

        let successes = tracker.commands_by_exit_code(0);
        assert_eq!(successes.len(), 2);

        let failures = tracker.commands_by_exit_code(1);
        assert_eq!(failures.len(), 1);
    }
}

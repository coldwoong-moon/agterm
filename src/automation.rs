//! Terminal Automation API
//!
//! This module provides a comprehensive automation framework for AgTerm, enabling
//! programmatic control of terminal sessions through scripts and commands.
//!
//! # Features
//!
//! - **Command System**: Send text, keys, and control sequences
//! - **Pattern Matching**: Wait for and expect specific output patterns
//! - **Screen Capture**: Capture terminal output at any time
//! - **Scripting DSL**: Simple domain-specific language for automation
//! - **Variable Support**: Use variables and environment variables in scripts
//! - **Conditional Execution**: Execute commands based on conditions
//!
//! # Example
//!
//! ```ignore
//! use agterm::automation::{AutomationEngine, AutomationScript};
//! use agterm::terminal::pty::PtyManager;
//!
//! // Create PTY session
//! let pty_manager = PtyManager::new();
//! let pty_id = pty_manager.create_session(80, 24).unwrap();
//!
//! // Parse and execute a script
//! let script_text = r#"
//!     SEND "echo hello"
//!     SEND_KEY Enter
//!     WAIT_FOR "hello" 5s
//!     CAPTURE
//! "#;
//!
//! let mut engine = AutomationEngine::new(pty_manager, pty_id);
//! let result = engine.execute_script_str(script_text);
//! ```

use crate::terminal::pty::{PtyId, PtyManager};
use regex::Regex;
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, error, info, instrument, warn};

/// Key codes that can be sent to the terminal
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    /// Enter/Return key
    Enter,
    /// Tab key
    Tab,
    /// Backspace key
    Backspace,
    /// Escape key
    Escape,
    /// Arrow keys
    Up,
    Down,
    Left,
    Right,
    /// Function keys F1-F12
    F(u8),
    /// Home key
    Home,
    /// End key
    End,
    /// Page Up key
    PageUp,
    /// Page Down key
    PageDown,
    /// Insert key
    Insert,
    /// Delete key
    Delete,
    /// Control + character (e.g., Ctrl+C)
    Ctrl(char),
    /// Alt + character
    Alt(char),
    /// Raw character
    Char(char),
}

impl Key {
    /// Convert key to terminal byte sequence
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Key::Enter => vec![b'\r'],
            Key::Tab => vec![b'\t'],
            Key::Backspace => vec![0x7F],
            Key::Escape => vec![0x1B],
            Key::Up => b"\x1B[A".to_vec(),
            Key::Down => b"\x1B[B".to_vec(),
            Key::Right => b"\x1B[C".to_vec(),
            Key::Left => b"\x1B[D".to_vec(),
            Key::F(n) if *n >= 1 && *n <= 4 => format!("\x1BO{}", (b'P' + n - 1) as char)
                .as_bytes()
                .to_vec(),
            Key::F(n) if *n >= 5 && *n <= 12 => {
                format!("\x1B[{}~", 15 + n - 5).as_bytes().to_vec()
            }
            Key::F(_) => vec![],
            Key::Home => b"\x1B[H".to_vec(),
            Key::End => b"\x1B[F".to_vec(),
            Key::PageUp => b"\x1B[5~".to_vec(),
            Key::PageDown => b"\x1B[6~".to_vec(),
            Key::Insert => b"\x1B[2~".to_vec(),
            Key::Delete => b"\x1B[3~".to_vec(),
            Key::Ctrl(c) => {
                let byte = (*c as u8) & 0x1F;
                vec![byte]
            }
            Key::Alt(c) => format!("\x1B{}", c).as_bytes().to_vec(),
            Key::Char(c) => c.to_string().as_bytes().to_vec(),
        }
    }

    /// Parse key from string representation
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "ENTER" | "RETURN" => Some(Key::Enter),
            "TAB" => Some(Key::Tab),
            "BACKSPACE" => Some(Key::Backspace),
            "ESCAPE" | "ESC" => Some(Key::Escape),
            "UP" => Some(Key::Up),
            "DOWN" => Some(Key::Down),
            "LEFT" => Some(Key::Left),
            "RIGHT" => Some(Key::Right),
            "HOME" => Some(Key::Home),
            "END" => Some(Key::End),
            "PAGEUP" => Some(Key::PageUp),
            "PAGEDOWN" => Some(Key::PageDown),
            "INSERT" => Some(Key::Insert),
            "DELETE" => Some(Key::Delete),
            s if s.starts_with('F') && s.len() <= 3 => {
                let num = s[1..].parse::<u8>().ok()?;
                if (1..=12).contains(&num) {
                    Some(Key::F(num))
                } else {
                    None
                }
            }
            s if s.starts_with("CTRL+") && s.len() == 6 => {
                let c = s.chars().nth(5)?;
                Some(Key::Ctrl(c))
            }
            s if s.starts_with("ALT+") && s.len() == 5 => {
                let c = s.chars().nth(4)?;
                Some(Key::Alt(c))
            }
            _ if s.len() == 1 => Some(Key::Char(s.chars().next()?)),
            _ => None,
        }
    }
}

/// Pattern for matching terminal output
#[derive(Debug, Clone)]
pub enum Pattern {
    /// Exact string match
    Exact(String),
    /// Regular expression match
    Regex(Regex),
    /// Any of multiple patterns
    AnyOf(Vec<Pattern>),
    /// All patterns must match
    AllOf(Vec<Pattern>),
}

impl Pattern {
    /// Check if pattern matches the given text
    pub fn matches(&self, text: &str) -> bool {
        match self {
            Pattern::Exact(s) => text.contains(s),
            Pattern::Regex(re) => re.is_match(text),
            Pattern::AnyOf(patterns) => patterns.iter().any(|p| p.matches(text)),
            Pattern::AllOf(patterns) => patterns.iter().all(|p| p.matches(text)),
        }
    }

    /// Extract matched text from the input
    pub fn extract(&self, text: &str) -> Option<String> {
        match self {
            Pattern::Exact(s) => {
                if text.contains(s) {
                    Some(s.clone())
                } else {
                    None
                }
            }
            Pattern::Regex(re) => re.find(text).map(|m| m.as_str().to_string()),
            Pattern::AnyOf(patterns) => patterns.iter().find_map(|p| p.extract(text)),
            Pattern::AllOf(patterns) => {
                if patterns.iter().all(|p| p.matches(text)) {
                    Some(text.to_string())
                } else {
                    None
                }
            }
        }
    }
}

/// Automation command that can be executed
#[derive(Debug, Clone)]
pub enum AutomationCommand {
    /// Send text to the terminal (with optional newline)
    SendText {
        text: String,
        append_newline: bool,
    },
    /// Send key sequence to the terminal
    SendKeys(Vec<Key>),
    /// Wait for a pattern to appear in output
    WaitFor {
        pattern: Pattern,
        timeout: Duration,
    },
    /// Capture current terminal output
    Capture {
        /// Variable name to store the captured output
        store_in: Option<String>,
    },
    /// Expect a pattern (fails if not found)
    Expect {
        pattern: Pattern,
        message: Option<String>,
    },
    /// Set a variable
    SetVariable {
        name: String,
        value: String,
    },
    /// Execute command if condition is true
    If {
        condition: Condition,
        then_commands: Vec<AutomationCommand>,
        else_commands: Vec<AutomationCommand>,
    },
    /// Wait for a specific duration
    Sleep(Duration),
    /// Clear the terminal screen
    Clear,
    /// Execute a raw command (shell command)
    Execute {
        command: String,
        wait: bool,
    },
}

/// Condition for conditional execution
#[derive(Debug, Clone)]
pub enum Condition {
    /// Variable equals value
    VarEquals(String, String),
    /// Variable contains substring
    VarContains(String, String),
    /// Variable matches regex
    VarMatches(String, Regex),
    /// Environment variable exists
    EnvExists(String),
    /// Pattern matches last captured output
    PatternMatches(Pattern),
    /// Logical AND
    And(Box<Condition>, Box<Condition>),
    /// Logical OR
    Or(Box<Condition>, Box<Condition>),
    /// Logical NOT
    Not(Box<Condition>),
}

impl Condition {
    /// Evaluate the condition
    pub fn evaluate(&self, context: &ExecutionContext) -> bool {
        match self {
            Condition::VarEquals(name, value) => {
                context.variables.get(name).map_or(false, |v| v == value)
            }
            Condition::VarContains(name, substring) => {
                context.variables.get(name).map_or(false, |v| v.contains(substring))
            }
            Condition::VarMatches(name, regex) => {
                context.variables.get(name).map_or(false, |v| regex.is_match(v))
            }
            Condition::EnvExists(name) => std::env::var(name).is_ok(),
            Condition::PatternMatches(pattern) => {
                context.last_capture.as_ref().map_or(false, |text| pattern.matches(text))
            }
            Condition::And(a, b) => a.evaluate(context) && b.evaluate(context),
            Condition::Or(a, b) => a.evaluate(context) || b.evaluate(context),
            Condition::Not(c) => !c.evaluate(context),
        }
    }
}

/// Result of automation command execution
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Whether the command succeeded
    pub success: bool,
    /// Output captured during execution (if any)
    pub output: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Duration of command execution
    pub duration: Duration,
}

/// Automation script containing multiple commands
#[derive(Debug, Clone)]
pub struct AutomationScript {
    /// Script name
    pub name: String,
    /// Commands to execute
    pub commands: Vec<AutomationCommand>,
    /// Script variables (key-value pairs)
    pub variables: HashMap<String, String>,
    /// Script description
    pub description: Option<String>,
}

impl AutomationScript {
    /// Create a new automation script
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            commands: Vec::new(),
            variables: HashMap::new(),
            description: None,
        }
    }

    /// Add a command to the script
    pub fn add_command(&mut self, command: AutomationCommand) {
        self.commands.push(command);
    }

    /// Set a variable in the script
    pub fn set_variable(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.variables.insert(name.into(), value.into());
    }

    /// Set script description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Get script name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get script description
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Get script commands
    pub fn commands(&self) -> &[AutomationCommand] {
        &self.commands
    }

    /// Get script variables
    pub fn variables(&self) -> &HashMap<String, String> {
        &self.variables
    }
}

/// Execution context for automation scripts
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Variables defined in the script
    pub variables: HashMap<String, String>,
    /// Last captured output
    pub last_capture: Option<String>,
    /// Output buffer for pattern matching
    pub output_buffer: String,
    /// Maximum output buffer size (to prevent memory issues)
    pub max_buffer_size: usize,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            variables: HashMap::new(),
            last_capture: None,
            output_buffer: String::new(),
            max_buffer_size: 1024 * 1024, // 1MB
        }
    }
}

impl ExecutionContext {
    /// Create a new execution context with initial variables
    pub fn new(variables: HashMap<String, String>) -> Self {
        Self {
            variables,
            last_capture: None,
            output_buffer: String::new(),
            max_buffer_size: 1024 * 1024,
        }
    }

    /// Add output to the buffer
    pub fn append_output(&mut self, output: &str) {
        self.output_buffer.push_str(output);

        // Trim buffer if it exceeds max size
        if self.output_buffer.len() > self.max_buffer_size {
            let trim_size = self.output_buffer.len() - self.max_buffer_size;
            self.output_buffer.drain(..trim_size);
        }
    }

    /// Expand variables in text (e.g., ${VAR_NAME} or $VAR_NAME)
    pub fn expand_variables(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Expand ${VAR} format
        for (name, value) in &self.variables {
            result = result.replace(&format!("${{{}}}", name), value);
            result = result.replace(&format!("${}", name), value);
        }

        // Expand environment variables with ENV: prefix
        let env_regex = Regex::new(r"\$\{ENV:([^}]+)\}").unwrap();
        for cap in env_regex.captures_iter(text) {
            if let Some(env_name) = cap.get(1) {
                if let Ok(env_value) = std::env::var(env_name.as_str()) {
                    result = result.replace(&cap[0], &env_value);
                }
            }
        }

        result
    }
}

/// Errors that can occur during automation
#[derive(Debug, Error)]
pub enum AutomationError {
    /// PTY operation failed
    #[error("PTY error: {0}")]
    PtyError(#[from] crate::terminal::pty::PtyError),

    /// Timeout while waiting for pattern
    #[error("Timeout waiting for pattern: {0}")]
    Timeout(String),

    /// Expected pattern not found
    #[error("Expected pattern not found: {0}")]
    ExpectationFailed(String),

    /// Invalid command syntax
    #[error("Invalid command syntax: {0}")]
    InvalidSyntax(String),

    /// Variable not found
    #[error("Variable not found: {0}")]
    VariableNotFound(String),

    /// Script parsing error
    #[error("Script parsing error at line {line}: {message}")]
    ParseError {
        line: usize,
        message: String,
    },

    /// General execution error
    #[error("Execution error: {0}")]
    ExecutionError(String),
}

/// Automation engine for executing scripts
pub struct AutomationEngine {
    /// PTY manager for terminal operations
    pty_manager: PtyManager,
    /// PTY session ID
    pty_id: PtyId,
    /// Execution context
    context: ExecutionContext,
}

impl AutomationEngine {
    /// Create a new automation engine
    pub fn new(pty_manager: PtyManager, pty_id: PtyId) -> Self {
        Self {
            pty_manager,
            pty_id,
            context: ExecutionContext::default(),
        }
    }

    /// Create engine with initial variables
    pub fn with_variables(
        pty_manager: PtyManager,
        pty_id: PtyId,
        variables: HashMap<String, String>,
    ) -> Self {
        Self {
            pty_manager,
            pty_id,
            context: ExecutionContext::new(variables),
        }
    }

    /// Execute a single command
    #[instrument(skip(self), fields(pty_id = %self.pty_id))]
    pub fn execute_command(&mut self, command: &AutomationCommand) -> Result<CommandResult, AutomationError> {
        let start = std::time::Instant::now();
        debug!("Executing command: {:?}", command);

        let result = match command {
            AutomationCommand::SendText { text, append_newline } => {
                let expanded = self.context.expand_variables(text);
                let mut data = expanded.as_bytes().to_vec();
                if *append_newline {
                    data.push(b'\n');
                }
                self.pty_manager.write(&self.pty_id, &data)?;
                CommandResult {
                    success: true,
                    output: None,
                    error: None,
                    duration: start.elapsed(),
                }
            }

            AutomationCommand::SendKeys(keys) => {
                let mut data = Vec::new();
                for key in keys {
                    data.extend(key.to_bytes());
                }
                self.pty_manager.write(&self.pty_id, &data)?;
                CommandResult {
                    success: true,
                    output: None,
                    error: None,
                    duration: start.elapsed(),
                }
            }

            AutomationCommand::WaitFor { pattern, timeout } => {
                self.wait_for_pattern(pattern, *timeout)?
            }

            AutomationCommand::Capture { store_in } => {
                let output = self.read_output()?;
                self.context.last_capture = Some(output.clone());

                if let Some(var_name) = store_in {
                    self.context.variables.insert(var_name.clone(), output.clone());
                }

                CommandResult {
                    success: true,
                    output: Some(output),
                    error: None,
                    duration: start.elapsed(),
                }
            }

            AutomationCommand::Expect { pattern, message } => {
                let output = self.read_output()?;
                if !pattern.matches(&output) {
                    let msg = message.clone().unwrap_or_else(|| "Pattern not found".to_string());
                    return Err(AutomationError::ExpectationFailed(msg));
                }
                CommandResult {
                    success: true,
                    output: Some(output),
                    error: None,
                    duration: start.elapsed(),
                }
            }

            AutomationCommand::SetVariable { name, value } => {
                let expanded = self.context.expand_variables(value);
                self.context.variables.insert(name.clone(), expanded);
                CommandResult {
                    success: true,
                    output: None,
                    error: None,
                    duration: start.elapsed(),
                }
            }

            AutomationCommand::If { condition, then_commands, else_commands } => {
                let commands = if condition.evaluate(&self.context) {
                    then_commands
                } else {
                    else_commands
                };

                for cmd in commands {
                    self.execute_command(cmd)?;
                }

                CommandResult {
                    success: true,
                    output: None,
                    error: None,
                    duration: start.elapsed(),
                }
            }

            AutomationCommand::Sleep(duration) => {
                std::thread::sleep(*duration);
                CommandResult {
                    success: true,
                    output: None,
                    error: None,
                    duration: start.elapsed(),
                }
            }

            AutomationCommand::Clear => {
                // Send clear screen sequence
                self.pty_manager.write(&self.pty_id, b"\x1B[2J\x1B[H")?;
                CommandResult {
                    success: true,
                    output: None,
                    error: None,
                    duration: start.elapsed(),
                }
            }

            AutomationCommand::Execute { command, wait } => {
                let expanded = self.context.expand_variables(command);
                let mut data = expanded.as_bytes().to_vec();
                data.push(b'\n');
                self.pty_manager.write(&self.pty_id, &data)?;

                if *wait {
                    std::thread::sleep(Duration::from_millis(100));
                    let output = self.read_output()?;
                    CommandResult {
                        success: true,
                        output: Some(output),
                        error: None,
                        duration: start.elapsed(),
                    }
                } else {
                    CommandResult {
                        success: true,
                        output: None,
                        error: None,
                        duration: start.elapsed(),
                    }
                }
            }
        };

        info!("Command executed successfully in {:?}", result.duration);
        Ok(result)
    }

    /// Execute a complete script
    #[instrument(skip(self, script), fields(script_name = %script.name))]
    pub fn execute_script(&mut self, script: &AutomationScript) -> Result<Vec<CommandResult>, AutomationError> {
        info!("Executing script: {}", script.name);

        // Initialize context with script variables
        for (name, value) in &script.variables {
            self.context.variables.insert(name.clone(), value.clone());
        }

        let mut results = Vec::new();

        for (idx, command) in script.commands.iter().enumerate() {
            match self.execute_command(command) {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    error!("Script failed at command {}: {:?}", idx, e);
                    return Err(e);
                }
            }
        }

        info!("Script completed successfully with {} commands", results.len());
        Ok(results)
    }

    /// Parse and execute a script from string
    pub fn execute_script_str(&mut self, script_text: &str) -> Result<Vec<CommandResult>, AutomationError> {
        let script = Self::parse_script(script_text)?;
        self.execute_script(&script)
    }

    /// Wait for a pattern to appear in output
    fn wait_for_pattern(&mut self, pattern: &Pattern, timeout: Duration) -> Result<CommandResult, AutomationError> {
        let start = std::time::Instant::now();
        let mut accumulated_output = String::new();

        loop {
            if start.elapsed() > timeout {
                return Err(AutomationError::Timeout(format!("{:?}", pattern)));
            }

            // Read available output
            if let Ok(output) = self.pty_manager.read(&self.pty_id) {
                if !output.is_empty() {
                    let text = String::from_utf8_lossy(&output);
                    accumulated_output.push_str(&text);
                    self.context.append_output(&text);

                    // Check if pattern matches
                    if pattern.matches(&accumulated_output) {
                        return Ok(CommandResult {
                            success: true,
                            output: Some(accumulated_output),
                            error: None,
                            duration: start.elapsed(),
                        });
                    }
                }
            }

            // Small delay to avoid busy-waiting
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// Read current output from PTY
    fn read_output(&mut self) -> Result<String, AutomationError> {
        match self.pty_manager.read(&self.pty_id) {
            Ok(output) => {
                let text = String::from_utf8_lossy(&output).to_string();
                self.context.append_output(&text);
                Ok(text)
            }
            Err(e) => Err(AutomationError::PtyError(e)),
        }
    }

    /// Parse a script from text using the DSL
    pub fn parse_script(text: &str) -> Result<AutomationScript, AutomationError> {
        let mut script = AutomationScript::new("parsed_script");
        let mut line_num = 0;

        for line in text.lines() {
            line_num += 1;
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let command = Self::parse_command_line(line, line_num)?;
            if let Some(cmd) = command {
                script.add_command(cmd);
            }
        }

        Ok(script)
    }

    /// Parse a single command line
    fn parse_command_line(line: &str, line_num: usize) -> Result<Option<AutomationCommand>, AutomationError> {
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        let command_name = parts[0].to_uppercase();

        match command_name.as_str() {
            "SEND" => {
                let text = Self::extract_string_arg(&parts, line_num)?;
                Ok(Some(AutomationCommand::SendText {
                    text,
                    append_newline: true,
                }))
            }

            "SEND_TEXT" => {
                let text = Self::extract_string_arg(&parts, line_num)?;
                Ok(Some(AutomationCommand::SendText {
                    text,
                    append_newline: false,
                }))
            }

            "SEND_KEY" => {
                let key_str = Self::extract_arg(&parts, line_num)?;
                let key = Key::from_str(key_str)
                    .ok_or_else(|| AutomationError::ParseError {
                        line: line_num,
                        message: format!("Invalid key: {}", key_str),
                    })?;
                Ok(Some(AutomationCommand::SendKeys(vec![key])))
            }

            "WAIT_FOR" => {
                let arg = Self::extract_arg(&parts, line_num)?;
                let (pattern_str, timeout_str) = if let Some(pos) = arg.find(' ') {
                    (&arg[..pos], &arg[pos + 1..])
                } else {
                    return Err(AutomationError::ParseError {
                        line: line_num,
                        message: "WAIT_FOR requires pattern and timeout".to_string(),
                    });
                };

                let pattern = Pattern::Exact(Self::unquote(pattern_str));
                let timeout = Self::parse_duration(timeout_str, line_num)?;

                Ok(Some(AutomationCommand::WaitFor { pattern, timeout }))
            }

            "CAPTURE" => {
                Ok(Some(AutomationCommand::Capture { store_in: None }))
            }

            "EXPECT" => {
                let pattern_str = Self::extract_string_arg(&parts, line_num)?;
                let pattern = Pattern::Exact(pattern_str);
                Ok(Some(AutomationCommand::Expect {
                    pattern,
                    message: None,
                }))
            }

            "SET" => {
                let arg = Self::extract_arg(&parts, line_num)?;
                let eq_pos = arg.find('=').ok_or_else(|| AutomationError::ParseError {
                    line: line_num,
                    message: "SET requires NAME=VALUE format".to_string(),
                })?;

                let name = arg[..eq_pos].trim().to_string();
                let value = Self::unquote(arg[eq_pos + 1..].trim());

                Ok(Some(AutomationCommand::SetVariable { name, value }))
            }

            "SLEEP" => {
                let duration_str = Self::extract_arg(&parts, line_num)?;
                let duration = Self::parse_duration(duration_str, line_num)?;
                Ok(Some(AutomationCommand::Sleep(duration)))
            }

            "CLEAR" => {
                Ok(Some(AutomationCommand::Clear))
            }

            "EXECUTE" => {
                let command = Self::extract_string_arg(&parts, line_num)?;
                Ok(Some(AutomationCommand::Execute {
                    command,
                    wait: false,
                }))
            }

            _ => {
                Err(AutomationError::ParseError {
                    line: line_num,
                    message: format!("Unknown command: {}", command_name),
                })
            }
        }
    }

    /// Extract argument from command parts
    fn extract_arg<'a>(parts: &'a [&'a str], line_num: usize) -> Result<&'a str, AutomationError> {
        parts.get(1).copied().ok_or_else(|| AutomationError::ParseError {
            line: line_num,
            message: "Missing argument".to_string(),
        })
    }

    /// Extract string argument (handles quotes)
    fn extract_string_arg(parts: &[&str], line_num: usize) -> Result<String, AutomationError> {
        let arg = Self::extract_arg(parts, line_num)?;
        Ok(Self::unquote(arg))
    }

    /// Remove quotes from string
    fn unquote(s: &str) -> String {
        let s = s.trim();
        if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
            s[1..s.len() - 1].to_string()
        } else {
            s.to_string()
        }
    }

    /// Parse duration string (e.g., "5s", "100ms", "2m")
    fn parse_duration(s: &str, line_num: usize) -> Result<Duration, AutomationError> {
        let s = s.trim();
        let (value_str, unit) = if s.ends_with("ms") {
            (&s[..s.len() - 2], "ms")
        } else if s.ends_with('s') {
            (&s[..s.len() - 1], "s")
        } else if s.ends_with('m') {
            (&s[..s.len() - 1], "m")
        } else {
            return Err(AutomationError::ParseError {
                line: line_num,
                message: format!("Invalid duration format: {}", s),
            });
        };

        let value: u64 = value_str.parse().map_err(|_| AutomationError::ParseError {
            line: line_num,
            message: format!("Invalid duration value: {}", value_str),
        })?;

        let duration = match unit {
            "ms" => Duration::from_millis(value),
            "s" => Duration::from_secs(value),
            "m" => Duration::from_secs(value * 60),
            _ => unreachable!(),
        };

        Ok(duration)
    }

    /// Get the current execution context
    pub fn context(&self) -> &ExecutionContext {
        &self.context
    }

    /// Get a mutable reference to the execution context
    pub fn context_mut(&mut self) -> &mut ExecutionContext {
        &mut self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_to_bytes() {
        assert_eq!(Key::Enter.to_bytes(), vec![b'\r']);
        assert_eq!(Key::Tab.to_bytes(), vec![b'\t']);
        assert_eq!(Key::Escape.to_bytes(), vec![0x1B]);
        assert_eq!(Key::Up.to_bytes(), b"\x1B[A");
        assert_eq!(Key::Ctrl('c').to_bytes(), vec![3]); // Ctrl+C = 0x03
        assert_eq!(Key::Char('a').to_bytes(), vec![b'a']);
    }

    #[test]
    fn test_key_from_str() {
        assert_eq!(Key::from_str("ENTER"), Some(Key::Enter));
        assert_eq!(Key::from_str("Tab"), Some(Key::Tab));
        assert_eq!(Key::from_str("F1"), Some(Key::F(1)));
        assert_eq!(Key::from_str("F12"), Some(Key::F(12)));
        assert_eq!(Key::from_str("CTRL+C"), Some(Key::Ctrl('C')));
        assert_eq!(Key::from_str("ALT+A"), Some(Key::Alt('A')));
        assert_eq!(Key::from_str("a"), Some(Key::Char('a')));
        assert_eq!(Key::from_str("INVALID"), None);
    }

    #[test]
    fn test_pattern_exact_match() {
        let pattern = Pattern::Exact("hello".to_string());
        assert!(pattern.matches("hello world"));
        assert!(pattern.matches("say hello"));
        assert!(!pattern.matches("goodbye"));
    }

    #[test]
    fn test_pattern_regex_match() {
        let pattern = Pattern::Regex(Regex::new(r"\d{3}-\d{4}").unwrap());
        assert!(pattern.matches("Call 555-1234 now"));
        assert!(!pattern.matches("No phone number here"));
    }

    #[test]
    fn test_pattern_any_of() {
        let pattern = Pattern::AnyOf(vec![
            Pattern::Exact("hello".to_string()),
            Pattern::Exact("goodbye".to_string()),
        ]);
        assert!(pattern.matches("hello world"));
        assert!(pattern.matches("goodbye world"));
        assert!(!pattern.matches("neither"));
    }

    #[test]
    fn test_context_variable_expansion() {
        let mut context = ExecutionContext::default();
        context.variables.insert("NAME".to_string(), "Alice".to_string());
        context.variables.insert("AGE".to_string(), "30".to_string());

        assert_eq!(
            context.expand_variables("Hello ${NAME}, you are $AGE years old"),
            "Hello Alice, you are 30 years old"
        );
    }

    #[test]
    fn test_context_env_expansion() {
        std::env::set_var("TEST_VAR", "test_value");
        let context = ExecutionContext::default();

        assert_eq!(
            context.expand_variables("Value: ${ENV:TEST_VAR}"),
            "Value: test_value"
        );
    }

    #[test]
    fn test_condition_var_equals() {
        let mut context = ExecutionContext::default();
        context.variables.insert("STATUS".to_string(), "ok".to_string());

        let condition = Condition::VarEquals("STATUS".to_string(), "ok".to_string());
        assert!(condition.evaluate(&context));

        let condition = Condition::VarEquals("STATUS".to_string(), "error".to_string());
        assert!(!condition.evaluate(&context));
    }

    #[test]
    fn test_condition_var_contains() {
        let mut context = ExecutionContext::default();
        context.variables.insert("OUTPUT".to_string(), "Error: file not found".to_string());

        let condition = Condition::VarContains("OUTPUT".to_string(), "Error".to_string());
        assert!(condition.evaluate(&context));

        let condition = Condition::VarContains("OUTPUT".to_string(), "Success".to_string());
        assert!(!condition.evaluate(&context));
    }

    #[test]
    fn test_condition_logical_ops() {
        let mut context = ExecutionContext::default();
        context.variables.insert("A".to_string(), "1".to_string());
        context.variables.insert("B".to_string(), "2".to_string());

        let cond_a = Condition::VarEquals("A".to_string(), "1".to_string());
        let cond_b = Condition::VarEquals("B".to_string(), "2".to_string());
        let cond_c = Condition::VarEquals("C".to_string(), "3".to_string());

        // AND
        let and_true = Condition::And(Box::new(cond_a.clone()), Box::new(cond_b.clone()));
        assert!(and_true.evaluate(&context));

        let and_false = Condition::And(Box::new(cond_a.clone()), Box::new(cond_c.clone()));
        assert!(!and_false.evaluate(&context));

        // OR
        let or_true = Condition::Or(Box::new(cond_a.clone()), Box::new(cond_c.clone()));
        assert!(or_true.evaluate(&context));

        // NOT
        let not = Condition::Not(Box::new(cond_c));
        assert!(not.evaluate(&context));
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(
            AutomationEngine::parse_duration("100ms", 1).unwrap(),
            Duration::from_millis(100)
        );
        assert_eq!(
            AutomationEngine::parse_duration("5s", 1).unwrap(),
            Duration::from_secs(5)
        );
        assert_eq!(
            AutomationEngine::parse_duration("2m", 1).unwrap(),
            Duration::from_secs(120)
        );
    }

    #[test]
    fn test_unquote() {
        assert_eq!(AutomationEngine::unquote("\"hello\""), "hello");
        assert_eq!(AutomationEngine::unquote("'world'"), "world");
        assert_eq!(AutomationEngine::unquote("no quotes"), "no quotes");
    }

    #[test]
    fn test_parse_script_basic() {
        let script_text = r#"
# This is a comment
SEND "echo hello"
SEND_KEY Enter
WAIT_FOR "hello" 5s
CAPTURE
EXPECT "hello"
        "#;

        let script = AutomationEngine::parse_script(script_text).unwrap();
        assert_eq!(script.commands.len(), 5);
    }

    #[test]
    fn test_parse_script_variables() {
        let script_text = r#"
SET NAME="Alice"
SEND "Hello ${NAME}"
        "#;

        let script = AutomationEngine::parse_script(script_text).unwrap();
        assert_eq!(script.commands.len(), 2);
    }

    #[test]
    fn test_automation_script_builder() {
        let mut script = AutomationScript::new("test_script")
            .with_description("A test script");

        script.set_variable("USER", "alice");
        script.add_command(AutomationCommand::SendText {
            text: "Hello ${USER}".to_string(),
            append_newline: true,
        });
        script.add_command(AutomationCommand::Sleep(Duration::from_millis(100)));

        assert_eq!(script.name, "test_script");
        assert_eq!(script.commands.len(), 2);
        assert_eq!(script.variables.get("USER"), Some(&"alice".to_string()));
    }
}

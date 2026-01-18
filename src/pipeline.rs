//! Command Pipeline System
//!
//! This module provides a comprehensive pipeline framework for AgTerm, enabling
//! the creation, management, and execution of command sequences with conditional
//! logic, error handling, and retry mechanisms.
//!
//! # Features
//!
//! - **Pipeline Steps**: Define command sequences with arguments
//! - **Conditional Execution**: Execute steps based on previous results
//! - **Error Handling**: Configure actions on step failure (continue, stop, retry, fallback)
//! - **Timeout Support**: Set timeouts for individual steps
//! - **Variable Substitution**: Use variables in commands and arguments
//! - **Pipeline Management**: Create, modify, clone, and delete pipelines
//! - **Pipeline Parsing**: Parse pipelines from shell-like syntax
//! - **Serialization**: Save and load pipelines from JSON files
//!
//! # Example
//!
//! ```ignore
//! use agterm::pipeline::{Pipeline, PipelineStep, PipelineManager, StepCondition, ErrorAction};
//! use std::time::Duration;
//!
//! // Create a new pipeline
//! let mut manager = PipelineManager::new();
//! let id = manager.create_pipeline("build-and-test", Some("Build and run tests"));
//!
//! // Add steps
//! manager.add_step(&id, PipelineStep {
//!     id: 0,
//!     command: "cargo".to_string(),
//!     args: vec!["build".to_string()],
//!     condition: Some(StepCondition::Always),
//!     on_error: ErrorAction::Stop,
//!     timeout: Some(Duration::from_secs(300)),
//! }).unwrap();
//!
//! manager.add_step(&id, PipelineStep {
//!     id: 1,
//!     command: "cargo".to_string(),
//!     args: vec!["test".to_string()],
//!     condition: Some(StepCondition::OnSuccess),
//!     on_error: ErrorAction::Stop,
//!     timeout: Some(Duration::from_secs(600)),
//! }).unwrap();
//!
//! // Or parse from string
//! let pipeline = Pipeline::parse_from_string("cargo build && cargo test").unwrap();
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, info};
use uuid::Uuid;

/// Condition for step execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepCondition {
    /// Always execute (default)
    Always,
    /// Execute only if previous step succeeded
    OnSuccess,
    /// Execute only if previous step failed
    OnFailure,
    /// Execute based on custom condition expression
    Custom(String),
}

impl Default for StepCondition {
    fn default() -> Self {
        Self::Always
    }
}

/// Action to take when a step fails
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorAction {
    /// Continue to next step
    Continue,
    /// Stop pipeline execution
    Stop,
    /// Retry the step with specified parameters
    Retry {
        /// Maximum number of retry attempts
        max_attempts: u32,
        /// Delay between retries
        #[serde(with = "duration_serde")]
        delay: Duration,
    },
    /// Execute fallback command on failure
    Fallback(String),
}

impl Default for ErrorAction {
    fn default() -> Self {
        Self::Stop
    }
}

/// Helper module for Duration serialization
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

/// Helper module for Option<Duration> serialization
mod option_duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match duration {
            Some(d) => Some(d.as_secs()).serialize(serializer),
            None => None::<u64>.serialize(serializer),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = Option::<u64>::deserialize(deserializer)?;
        Ok(secs.map(Duration::from_secs))
    }
}

/// A single step in a pipeline
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipelineStep {
    /// Unique step identifier within the pipeline
    pub id: usize,
    /// Command to execute
    pub command: String,
    /// Command arguments
    pub args: Vec<String>,
    /// Execution condition
    #[serde(default)]
    pub condition: Option<StepCondition>,
    /// Action to take on error
    #[serde(default)]
    pub on_error: ErrorAction,
    /// Optional timeout for this step
    #[serde(with = "option_duration_serde")]
    pub timeout: Option<Duration>,
}

impl PipelineStep {
    /// Create a new pipeline step
    pub fn new(
        id: usize,
        command: impl Into<String>,
        args: Vec<String>,
    ) -> Self {
        Self {
            id,
            command: command.into(),
            args,
            condition: Some(StepCondition::Always),
            on_error: ErrorAction::Stop,
            timeout: None,
        }
    }

    /// Set execution condition
    pub fn with_condition(mut self, condition: StepCondition) -> Self {
        self.condition = Some(condition);
        self
    }

    /// Set error action
    pub fn with_error_action(mut self, on_error: ErrorAction) -> Self {
        self.on_error = on_error;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Get full command string (command + args)
    pub fn full_command(&self) -> String {
        if self.args.is_empty() {
            self.command.clone()
        } else {
            format!("{} {}", self.command, self.args.join(" "))
        }
    }
}

/// Result of a single step execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// Step identifier
    pub step_id: usize,
    /// Exit code (0 = success, non-zero = error)
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Execution duration
    #[serde(with = "duration_serde")]
    pub duration: Duration,
    /// Whether the step succeeded
    pub success: bool,
}

impl StepResult {
    /// Create a new step result
    pub fn new(
        step_id: usize,
        exit_code: i32,
        stdout: String,
        stderr: String,
        duration: Duration,
    ) -> Self {
        Self {
            step_id,
            exit_code,
            stdout,
            stderr,
            duration,
            success: exit_code == 0,
        }
    }

    /// Create a successful result
    pub fn success(step_id: usize, stdout: String, duration: Duration) -> Self {
        Self::new(step_id, 0, stdout, String::new(), duration)
    }

    /// Create a failed result
    pub fn failure(step_id: usize, stderr: String, duration: Duration) -> Self {
        Self::new(step_id, 1, String::new(), stderr, duration)
    }
}

/// Result of a complete pipeline execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    /// Pipeline identifier
    pub pipeline_id: Uuid,
    /// Results from each executed step
    pub step_results: Vec<StepResult>,
    /// Total execution duration
    #[serde(with = "duration_serde")]
    pub total_duration: Duration,
    /// Whether the pipeline succeeded overall
    pub success: bool,
}

impl PipelineResult {
    /// Create a new pipeline result
    pub fn new(
        pipeline_id: Uuid,
        step_results: Vec<StepResult>,
        total_duration: Duration,
    ) -> Self {
        let success = step_results.iter().all(|r| r.success);
        Self {
            pipeline_id,
            step_results,
            total_duration,
            success,
        }
    }

    /// Get result for a specific step
    pub fn get_step_result(&self, step_id: usize) -> Option<&StepResult> {
        self.step_results.iter().find(|r| r.step_id == step_id)
    }

    /// Get all failed steps
    pub fn failed_steps(&self) -> Vec<&StepResult> {
        self.step_results.iter().filter(|r| !r.success).collect()
    }

    /// Get total number of steps executed
    pub fn steps_executed(&self) -> usize {
        self.step_results.len()
    }
}

/// A command pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    /// Unique pipeline identifier
    pub id: Uuid,
    /// Pipeline name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Pipeline steps in execution order
    pub steps: Vec<PipelineStep>,
    /// Variables for substitution in commands
    pub variables: HashMap<String, String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

impl Pipeline {
    /// Create a new pipeline
    pub fn new(name: impl Into<String>, description: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description,
            steps: Vec::new(),
            variables: HashMap::new(),
            created_at: Utc::now(),
        }
    }

    /// Add a step to the pipeline
    pub fn add_step(&mut self, mut step: PipelineStep) {
        step.id = self.steps.len();
        self.steps.push(step);
    }

    /// Remove a step by ID
    pub fn remove_step(&mut self, step_id: usize) -> Option<PipelineStep> {
        if step_id < self.steps.len() {
            let step = self.steps.remove(step_id);
            // Re-index remaining steps
            for (idx, s) in self.steps.iter_mut().enumerate().skip(step_id) {
                s.id = idx;
            }
            Some(step)
        } else {
            None
        }
    }

    /// Reorder steps
    pub fn reorder_steps(&mut self, new_order: Vec<usize>) -> Result<(), PipelineError> {
        if new_order.len() != self.steps.len() {
            return Err(PipelineError::InvalidOperation(
                "New order must contain all step indices".to_string(),
            ));
        }

        // Check for duplicates and valid indices
        let mut seen = vec![false; self.steps.len()];
        for &idx in &new_order {
            if idx >= self.steps.len() {
                return Err(PipelineError::InvalidOperation(format!(
                    "Invalid step index: {idx}"
                )));
            }
            if seen[idx] {
                return Err(PipelineError::InvalidOperation(format!(
                    "Duplicate step index: {idx}"
                )));
            }
            seen[idx] = true;
        }

        // Create new step order
        let mut new_steps = Vec::new();
        for &idx in &new_order {
            let mut step = self.steps[idx].clone();
            step.id = new_steps.len();
            new_steps.push(step);
        }

        self.steps = new_steps;
        Ok(())
    }

    /// Set a variable
    pub fn set_variable(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.variables.insert(key.into(), value.into());
    }

    /// Get a variable value
    pub fn get_variable(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }

    /// Expand variables in a string (${VAR} or $VAR format)
    pub fn expand_variables(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Expand ${VAR} and $VAR format
        for (name, value) in &self.variables {
            result = result.replace(&format!("${{{name}}}"), value);
            result = result.replace(&format!("${name}"), value);
        }

        result
    }

    /// Parse a pipeline from shell-like syntax
    ///
    /// Supported formats:
    /// - `cmd1 | cmd2 | cmd3` - Sequential execution (pipe-like)
    /// - `cmd1 && cmd2 && cmd3` - Execute next only if previous succeeded
    /// - `cmd1 || cmd2 || cmd3` - Execute next only if previous failed
    /// - `cmd1; cmd2; cmd3` - Always execute next (continue on error)
    pub fn parse_from_string(input: &str) -> Result<Self, PipelineError> {
        let mut pipeline = Pipeline::new("parsed", None);
        let input = input.trim();

        if input.is_empty() {
            return Err(PipelineError::ParseError("Empty pipeline string".to_string()));
        }

        // Determine separator type and split
        let (commands, condition, error_action) = if input.contains("&&") {
            (
                input.split("&&").collect::<Vec<_>>(),
                StepCondition::OnSuccess,
                ErrorAction::Stop,
            )
        } else if input.contains("||") {
            (
                input.split("||").collect::<Vec<_>>(),
                StepCondition::OnFailure,
                ErrorAction::Continue,
            )
        } else if input.contains(';') {
            (
                input.split(';').collect::<Vec<_>>(),
                StepCondition::Always,
                ErrorAction::Continue,
            )
        } else if input.contains('|') {
            (
                input.split('|').collect::<Vec<_>>(),
                StepCondition::OnSuccess,
                ErrorAction::Stop,
            )
        } else {
            // Single command
            (vec![input], StepCondition::Always, ErrorAction::Stop)
        };

        for (idx, cmd) in commands.iter().enumerate() {
            let cmd = cmd.trim();
            if cmd.is_empty() {
                continue;
            }

            // Parse command and arguments
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let command = parts[0].to_string();
            let args = parts[1..].iter().map(|s| s.to_string()).collect();

            let step_condition = if idx == 0 {
                StepCondition::Always
            } else {
                condition.clone()
            };

            let step = PipelineStep {
                id: pipeline.steps.len(),
                command,
                args,
                condition: Some(step_condition),
                on_error: error_action.clone(),
                timeout: None,
            };

            pipeline.add_step(step);
        }

        if pipeline.steps.is_empty() {
            return Err(PipelineError::ParseError(
                "No valid commands found".to_string(),
            ));
        }

        Ok(pipeline)
    }

    /// Save pipeline to a JSON file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), PipelineError> {
        let file = File::create(path)
            .map_err(|e| PipelineError::IoError(e.to_string()))?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)
            .map_err(|e| PipelineError::SerializationError(e.to_string()))?;
        Ok(())
    }

    /// Load pipeline from a JSON file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, PipelineError> {
        let file = File::open(path)
            .map_err(|e| PipelineError::IoError(e.to_string()))?;
        let reader = BufReader::new(file);
        let pipeline = serde_json::from_reader(reader)
            .map_err(|e| PipelineError::SerializationError(e.to_string()))?;
        Ok(pipeline)
    }
}

/// Pipeline manager for creating and managing multiple pipelines
#[derive(Debug, Default)]
pub struct PipelineManager {
    /// All managed pipelines
    pipelines: HashMap<Uuid, Pipeline>,
}

impl PipelineManager {
    /// Create a new pipeline manager
    pub fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
        }
    }

    /// Create a new pipeline and return its ID
    pub fn create_pipeline(
        &mut self,
        name: impl Into<String>,
        description: Option<String>,
    ) -> Uuid {
        let pipeline = Pipeline::new(name, description);
        let id = pipeline.id;
        self.pipelines.insert(id, pipeline);
        info!("Created pipeline: {}", id);
        id
    }

    /// Add a step to a pipeline
    pub fn add_step(&mut self, pipeline_id: &Uuid, step: PipelineStep) -> Result<(), PipelineError> {
        let pipeline = self.pipelines.get_mut(pipeline_id)
            .ok_or(PipelineError::NotFound(*pipeline_id))?;
        pipeline.add_step(step);
        debug!("Added step to pipeline {}", pipeline_id);
        Ok(())
    }

    /// Remove a step from a pipeline
    pub fn remove_step(
        &mut self,
        pipeline_id: &Uuid,
        step_id: usize,
    ) -> Result<PipelineStep, PipelineError> {
        let pipeline = self.pipelines.get_mut(pipeline_id)
            .ok_or(PipelineError::NotFound(*pipeline_id))?;
        pipeline.remove_step(step_id)
            .ok_or_else(|| PipelineError::InvalidOperation(format!("Step {step_id} not found")))
    }

    /// Reorder steps in a pipeline
    pub fn reorder_steps(
        &mut self,
        pipeline_id: &Uuid,
        new_order: Vec<usize>,
    ) -> Result<(), PipelineError> {
        let pipeline = self.pipelines.get_mut(pipeline_id)
            .ok_or(PipelineError::NotFound(*pipeline_id))?;
        pipeline.reorder_steps(new_order)
    }

    /// Get a reference to a pipeline
    pub fn get_pipeline(&self, id: &Uuid) -> Option<&Pipeline> {
        self.pipelines.get(id)
    }

    /// Get a mutable reference to a pipeline
    pub fn get_pipeline_mut(&mut self, id: &Uuid) -> Option<&mut Pipeline> {
        self.pipelines.get_mut(id)
    }

    /// List all pipelines
    pub fn list_pipelines(&self) -> Vec<&Pipeline> {
        self.pipelines.values().collect()
    }

    /// Delete a pipeline
    pub fn delete_pipeline(&mut self, id: &Uuid) -> Result<Pipeline, PipelineError> {
        self.pipelines.remove(id)
            .ok_or(PipelineError::NotFound(*id))
    }

    /// Clone a pipeline with a new name
    pub fn clone_pipeline(
        &mut self,
        id: &Uuid,
        new_name: impl Into<String>,
    ) -> Result<Uuid, PipelineError> {
        let pipeline = self.pipelines.get(id)
            .ok_or(PipelineError::NotFound(*id))?;

        let mut new_pipeline = pipeline.clone();
        new_pipeline.id = Uuid::new_v4();
        new_pipeline.name = new_name.into();
        new_pipeline.created_at = Utc::now();

        let new_id = new_pipeline.id;
        self.pipelines.insert(new_id, new_pipeline);

        info!("Cloned pipeline {} to {}", id, new_id);
        Ok(new_id)
    }

    /// Import a pipeline from JSON file
    pub fn import_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<Uuid, PipelineError> {
        let mut pipeline = Pipeline::load_from_file(path)?;

        // Generate new ID to avoid conflicts
        let old_id = pipeline.id;
        pipeline.id = Uuid::new_v4();

        let new_id = pipeline.id;
        self.pipelines.insert(new_id, pipeline);

        info!("Imported pipeline {} as {}", old_id, new_id);
        Ok(new_id)
    }

    /// Export a pipeline to JSON file
    pub fn export_to_file<P: AsRef<Path>>(
        &self,
        pipeline_id: &Uuid,
        path: P,
    ) -> Result<(), PipelineError> {
        let pipeline = self.pipelines.get(pipeline_id)
            .ok_or(PipelineError::NotFound(*pipeline_id))?;
        pipeline.save_to_file(path)
    }

    /// Get number of managed pipelines
    pub fn len(&self) -> usize {
        self.pipelines.len()
    }

    /// Check if manager has no pipelines
    pub fn is_empty(&self) -> bool {
        self.pipelines.is_empty()
    }
}

/// Errors that can occur in pipeline operations
#[derive(Debug, Error)]
pub enum PipelineError {
    /// Pipeline not found
    #[error("Pipeline not found: {0}")]
    NotFound(Uuid),

    /// Invalid operation
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Execution error
    #[error("Execution error: {0}")]
    ExecutionError(String),

    /// Timeout error
    #[error("Step timed out: {0}")]
    Timeout(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        let pipeline = Pipeline::new("test", Some("Test pipeline".to_string()));
        assert_eq!(pipeline.name, "test");
        assert_eq!(pipeline.description, Some("Test pipeline".to_string()));
        assert!(pipeline.steps.is_empty());
        assert!(pipeline.variables.is_empty());
    }

    #[test]
    fn test_add_and_remove_steps() {
        let mut pipeline = Pipeline::new("test", None);

        let step1 = PipelineStep::new(0, "echo", vec!["hello".to_string()]);
        let step2 = PipelineStep::new(0, "ls", vec!["-la".to_string()]);

        pipeline.add_step(step1);
        pipeline.add_step(step2);

        assert_eq!(pipeline.steps.len(), 2);
        assert_eq!(pipeline.steps[0].id, 0);
        assert_eq!(pipeline.steps[1].id, 1);

        let removed = pipeline.remove_step(0).unwrap();
        assert_eq!(removed.command, "echo");
        assert_eq!(pipeline.steps.len(), 1);
        assert_eq!(pipeline.steps[0].id, 0); // Re-indexed
        assert_eq!(pipeline.steps[0].command, "ls");
    }

    #[test]
    fn test_reorder_steps() {
        let mut pipeline = Pipeline::new("test", None);

        pipeline.add_step(PipelineStep::new(0, "cmd1", vec![]));
        pipeline.add_step(PipelineStep::new(0, "cmd2", vec![]));
        pipeline.add_step(PipelineStep::new(0, "cmd3", vec![]));

        pipeline.reorder_steps(vec![2, 0, 1]).unwrap();

        assert_eq!(pipeline.steps[0].command, "cmd3");
        assert_eq!(pipeline.steps[1].command, "cmd1");
        assert_eq!(pipeline.steps[2].command, "cmd2");

        // Check re-indexing
        assert_eq!(pipeline.steps[0].id, 0);
        assert_eq!(pipeline.steps[1].id, 1);
        assert_eq!(pipeline.steps[2].id, 2);
    }

    #[test]
    fn test_reorder_steps_invalid() {
        let mut pipeline = Pipeline::new("test", None);
        pipeline.add_step(PipelineStep::new(0, "cmd1", vec![]));

        // Wrong length
        assert!(pipeline.reorder_steps(vec![0, 1]).is_err());

        // Duplicate index
        pipeline.add_step(PipelineStep::new(0, "cmd2", vec![]));
        assert!(pipeline.reorder_steps(vec![0, 0]).is_err());

        // Invalid index
        assert!(pipeline.reorder_steps(vec![0, 5]).is_err());
    }

    #[test]
    fn test_variable_expansion() {
        let mut pipeline = Pipeline::new("test", None);
        pipeline.set_variable("NAME", "world");
        pipeline.set_variable("COUNT", "42");

        assert_eq!(
            pipeline.expand_variables("Hello ${NAME}!"),
            "Hello world!"
        );
        assert_eq!(
            pipeline.expand_variables("Count: $COUNT"),
            "Count: 42"
        );
        assert_eq!(
            pipeline.expand_variables("${NAME} ${COUNT}"),
            "world 42"
        );
    }

    #[test]
    fn test_parse_pipe_syntax() {
        let pipeline = Pipeline::parse_from_string("cat file.txt | grep pattern | wc -l").unwrap();

        assert_eq!(pipeline.steps.len(), 3);
        assert_eq!(pipeline.steps[0].command, "cat");
        assert_eq!(pipeline.steps[0].args, vec!["file.txt"]);
        assert_eq!(pipeline.steps[1].command, "grep");
        assert_eq!(pipeline.steps[1].args, vec!["pattern"]);
        assert_eq!(pipeline.steps[2].command, "wc");
        assert_eq!(pipeline.steps[2].args, vec!["-l"]);

        // Check conditions
        assert_eq!(pipeline.steps[0].condition, Some(StepCondition::Always));
        assert_eq!(pipeline.steps[1].condition, Some(StepCondition::OnSuccess));
        assert_eq!(pipeline.steps[2].condition, Some(StepCondition::OnSuccess));
    }

    #[test]
    fn test_parse_and_syntax() {
        let pipeline = Pipeline::parse_from_string("make && make test && make install").unwrap();

        assert_eq!(pipeline.steps.len(), 3);
        assert_eq!(pipeline.steps[0].command, "make");
        assert_eq!(pipeline.steps[1].command, "make");
        assert_eq!(pipeline.steps[1].args, vec!["test"]);
        assert_eq!(pipeline.steps[2].command, "make");
        assert_eq!(pipeline.steps[2].args, vec!["install"]);

        // All steps should have OnSuccess condition except first
        assert_eq!(pipeline.steps[0].condition, Some(StepCondition::Always));
        assert_eq!(pipeline.steps[1].condition, Some(StepCondition::OnSuccess));
        assert_eq!(pipeline.steps[2].condition, Some(StepCondition::OnSuccess));

        // Should stop on error
        assert_eq!(pipeline.steps[0].on_error, ErrorAction::Stop);
    }

    #[test]
    fn test_parse_or_syntax() {
        let pipeline = Pipeline::parse_from_string("cmd1 || cmd2 || cmd3").unwrap();

        assert_eq!(pipeline.steps.len(), 3);
        assert_eq!(pipeline.steps[0].condition, Some(StepCondition::Always));
        assert_eq!(pipeline.steps[1].condition, Some(StepCondition::OnFailure));
        assert_eq!(pipeline.steps[2].condition, Some(StepCondition::OnFailure));

        // Should continue on error
        assert_eq!(pipeline.steps[0].on_error, ErrorAction::Continue);
    }

    #[test]
    fn test_parse_semicolon_syntax() {
        let pipeline = Pipeline::parse_from_string("cmd1; cmd2; cmd3").unwrap();

        assert_eq!(pipeline.steps.len(), 3);
        assert_eq!(pipeline.steps[0].condition, Some(StepCondition::Always));
        assert_eq!(pipeline.steps[1].condition, Some(StepCondition::Always));
        assert_eq!(pipeline.steps[2].condition, Some(StepCondition::Always));

        // Should continue on error
        assert_eq!(pipeline.steps[0].on_error, ErrorAction::Continue);
    }

    #[test]
    fn test_parse_single_command() {
        let pipeline = Pipeline::parse_from_string("echo hello world").unwrap();

        assert_eq!(pipeline.steps.len(), 1);
        assert_eq!(pipeline.steps[0].command, "echo");
        assert_eq!(pipeline.steps[0].args, vec!["hello", "world"]);
        assert_eq!(pipeline.steps[0].condition, Some(StepCondition::Always));
    }

    #[test]
    fn test_parse_empty_error() {
        assert!(Pipeline::parse_from_string("").is_err());
        assert!(Pipeline::parse_from_string("   ").is_err());
    }

    #[test]
    fn test_step_full_command() {
        let step = PipelineStep::new(0, "git", vec!["commit".to_string(), "-m".to_string(), "message".to_string()]);
        assert_eq!(step.full_command(), "git commit -m message");

        let step = PipelineStep::new(0, "ls", vec![]);
        assert_eq!(step.full_command(), "ls");
    }

    #[test]
    fn test_step_builder() {
        let step = PipelineStep::new(0, "test", vec![])
            .with_condition(StepCondition::OnSuccess)
            .with_error_action(ErrorAction::Retry {
                max_attempts: 3,
                delay: Duration::from_secs(1),
            })
            .with_timeout(Duration::from_secs(30));

        assert_eq!(step.condition, Some(StepCondition::OnSuccess));
        assert_eq!(
            step.on_error,
            ErrorAction::Retry {
                max_attempts: 3,
                delay: Duration::from_secs(1),
            }
        );
        assert_eq!(step.timeout, Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_step_result() {
        let result = StepResult::success(0, "output".to_string(), Duration::from_secs(1));
        assert!(result.success);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "output");

        let result = StepResult::failure(1, "error".to_string(), Duration::from_secs(2));
        assert!(!result.success);
        assert_eq!(result.exit_code, 1);
        assert_eq!(result.stderr, "error");
    }

    #[test]
    fn test_pipeline_result() {
        let results = vec![
            StepResult::success(0, "out1".to_string(), Duration::from_secs(1)),
            StepResult::success(1, "out2".to_string(), Duration::from_secs(2)),
        ];

        let pipeline_result = PipelineResult::new(
            Uuid::new_v4(),
            results,
            Duration::from_secs(3),
        );

        assert!(pipeline_result.success);
        assert_eq!(pipeline_result.steps_executed(), 2);
        assert_eq!(pipeline_result.failed_steps().len(), 0);

        let result = pipeline_result.get_step_result(1).unwrap();
        assert_eq!(result.stdout, "out2");
    }

    #[test]
    fn test_pipeline_result_with_failure() {
        let results = vec![
            StepResult::success(0, "out1".to_string(), Duration::from_secs(1)),
            StepResult::failure(1, "error".to_string(), Duration::from_secs(2)),
        ];

        let pipeline_result = PipelineResult::new(
            Uuid::new_v4(),
            results,
            Duration::from_secs(3),
        );

        assert!(!pipeline_result.success);
        assert_eq!(pipeline_result.failed_steps().len(), 1);
        assert_eq!(pipeline_result.failed_steps()[0].stderr, "error");
    }

    #[test]
    fn test_pipeline_manager_create() {
        let mut manager = PipelineManager::new();
        let id = manager.create_pipeline("test", Some("desc".to_string()));

        assert_eq!(manager.len(), 1);

        let pipeline = manager.get_pipeline(&id).unwrap();
        assert_eq!(pipeline.name, "test");
        assert_eq!(pipeline.description, Some("desc".to_string()));
    }

    #[test]
    fn test_pipeline_manager_operations() {
        let mut manager = PipelineManager::new();
        let id = manager.create_pipeline("test", None);

        let step = PipelineStep::new(0, "echo", vec!["hello".to_string()]);
        manager.add_step(&id, step).unwrap();

        let pipeline = manager.get_pipeline(&id).unwrap();
        assert_eq!(pipeline.steps.len(), 1);

        manager.remove_step(&id, 0).unwrap();
        let pipeline = manager.get_pipeline(&id).unwrap();
        assert_eq!(pipeline.steps.len(), 0);
    }

    #[test]
    fn test_pipeline_manager_clone() {
        let mut manager = PipelineManager::new();
        let id = manager.create_pipeline("original", None);

        let step = PipelineStep::new(0, "test", vec![]);
        manager.add_step(&id, step).unwrap();

        let new_id = manager.clone_pipeline(&id, "cloned").unwrap();

        assert_eq!(manager.len(), 2);
        assert_ne!(id, new_id);

        let original = manager.get_pipeline(&id).unwrap();
        let cloned = manager.get_pipeline(&new_id).unwrap();

        assert_eq!(original.name, "original");
        assert_eq!(cloned.name, "cloned");
        assert_eq!(original.steps.len(), cloned.steps.len());
    }

    #[test]
    fn test_pipeline_manager_delete() {
        let mut manager = PipelineManager::new();
        let id = manager.create_pipeline("test", None);

        assert_eq!(manager.len(), 1);

        manager.delete_pipeline(&id).unwrap();

        assert_eq!(manager.len(), 0);
        assert!(manager.get_pipeline(&id).is_none());
    }

    #[test]
    fn test_pipeline_manager_list() {
        let mut manager = PipelineManager::new();
        manager.create_pipeline("test1", None);
        manager.create_pipeline("test2", None);
        manager.create_pipeline("test3", None);

        let pipelines = manager.list_pipelines();
        assert_eq!(pipelines.len(), 3);
    }

    #[test]
    fn test_pipeline_serialization() {
        let mut pipeline = Pipeline::new("test", Some("Test pipeline".to_string()));
        pipeline.set_variable("VAR", "value");

        let step = PipelineStep::new(0, "echo", vec!["${VAR}".to_string()])
            .with_timeout(Duration::from_secs(10));
        pipeline.add_step(step);

        // Serialize to JSON
        let json = serde_json::to_string(&pipeline).unwrap();

        // Deserialize back
        let deserialized: Pipeline = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, pipeline.name);
        assert_eq!(deserialized.description, pipeline.description);
        assert_eq!(deserialized.steps.len(), pipeline.steps.len());
        assert_eq!(deserialized.variables, pipeline.variables);
    }

    #[test]
    fn test_pipeline_file_operations() {
        use tempfile::NamedTempFile;

        let mut pipeline = Pipeline::new("test", Some("desc".to_string()));
        pipeline.add_step(PipelineStep::new(0, "echo", vec!["hello".to_string()]));

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Save
        pipeline.save_to_file(path).unwrap();

        // Load
        let loaded = Pipeline::load_from_file(path).unwrap();

        assert_eq!(loaded.name, "test");
        assert_eq!(loaded.description, Some("desc".to_string()));
        assert_eq!(loaded.steps.len(), 1);
        assert_eq!(loaded.steps[0].command, "echo");
    }

    #[test]
    fn test_pipeline_manager_import_export() {
        use tempfile::NamedTempFile;

        let mut manager = PipelineManager::new();
        let id = manager.create_pipeline("test", None);
        manager.add_step(&id, PipelineStep::new(0, "test", vec![])).unwrap();

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Export
        manager.export_to_file(&id, path).unwrap();

        // Import
        let new_id = manager.import_from_file(path).unwrap();

        assert_ne!(id, new_id);
        assert_eq!(manager.len(), 2);

        let imported = manager.get_pipeline(&new_id).unwrap();
        assert_eq!(imported.name, "test");
        assert_eq!(imported.steps.len(), 1);
    }

    #[test]
    fn test_error_action_retry() {
        let action = ErrorAction::Retry {
            max_attempts: 5,
            delay: Duration::from_secs(2),
        };

        match action {
            ErrorAction::Retry { max_attempts, delay } => {
                assert_eq!(max_attempts, 5);
                assert_eq!(delay, Duration::from_secs(2));
            }
            _ => panic!("Expected Retry variant"),
        }
    }

    #[test]
    fn test_error_action_fallback() {
        let action = ErrorAction::Fallback("alternative_command".to_string());

        match action {
            ErrorAction::Fallback(cmd) => {
                assert_eq!(cmd, "alternative_command");
            }
            _ => panic!("Expected Fallback variant"),
        }
    }

    #[test]
    fn test_step_condition_custom() {
        let condition = StepCondition::Custom("exit_code == 0".to_string());

        match condition {
            StepCondition::Custom(expr) => {
                assert_eq!(expr, "exit_code == 0");
            }
            _ => panic!("Expected Custom variant"),
        }
    }
}

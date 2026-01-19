//! Execution Pipeline Module
//!
//! This module provides a command execution pipeline with risk assessment,
//! approval workflow, and execution tracking for AI agent commands.

use chrono::{DateTime, Utc};
use floem::reactive::{RwSignal, SignalGet, SignalUpdate};
use uuid::Uuid;

use crate::floem_app::async_bridge::RiskLevel;

/// Pipeline stage for a command
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineStage {
    /// Command received from agent
    Received,
    /// Risk assessment completed
    Validated,
    /// Waiting for user approval
    PendingApproval,
    /// Approved by user
    Approved,
    /// Currently executing
    Executing,
    /// Execution completed successfully
    Completed,
    /// Execution failed
    Failed,
    /// Cancelled by user
    Cancelled,
}

/// A single item in the execution pipeline
#[derive(Debug, Clone)]
pub struct PipelineItem {
    /// Unique identifier
    pub id: Uuid,
    /// Agent that requested this command
    pub agent_id: String,
    /// Command to execute
    pub command: String,
    /// Optional description of what the command does
    pub description: Option<String>,
    /// Risk level assessment
    pub risk_level: RiskLevel,
    /// Current pipeline stage
    pub stage: RwSignal<PipelineStage>,
    /// When this item was created
    pub created_at: DateTime<Utc>,
    /// Output from execution (if completed)
    pub output: RwSignal<Option<String>>,
    /// Exit code from execution (if completed)
    pub exit_code: RwSignal<Option<i32>>,
}

impl PipelineItem {
    /// Create a new pipeline item
    pub fn new(
        agent_id: String,
        command: String,
        description: Option<String>,
        risk_level: RiskLevel,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            agent_id,
            command,
            description,
            risk_level,
            stage: RwSignal::new(PipelineStage::Received),
            created_at: Utc::now(),
            output: RwSignal::new(None),
            exit_code: RwSignal::new(None),
        }
    }

    /// Check if this item is in a terminal state (completed, failed, or cancelled)
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.stage.get(),
            PipelineStage::Completed | PipelineStage::Failed | PipelineStage::Cancelled
        )
    }

    /// Check if this item is pending approval
    pub fn is_pending_approval(&self) -> bool {
        self.stage.get() == PipelineStage::PendingApproval
    }

    /// Check if this item is currently executing
    pub fn is_executing(&self) -> bool {
        self.stage.get() == PipelineStage::Executing
    }
}

/// The execution pipeline manages command execution with approval workflow
#[derive(Clone)]
pub struct ExecutionPipeline {
    /// All pipeline items
    pub items: RwSignal<Vec<PipelineItem>>,
    /// Auto-approve commands at or below this risk level
    pub auto_approve_level: RwSignal<RiskLevel>,
}

impl ExecutionPipeline {
    /// Create a new execution pipeline
    pub fn new() -> Self {
        Self {
            items: RwSignal::new(Vec::new()),
            auto_approve_level: RwSignal::new(RiskLevel::Low),
        }
    }

    /// Enqueue a new command for execution
    ///
    /// Returns the unique ID of the pipeline item
    pub fn enqueue(
        &self,
        agent_id: String,
        command: String,
        description: Option<String>,
        risk_level: RiskLevel,
    ) -> Uuid {
        let item = PipelineItem::new(agent_id, command, description, risk_level);
        let id = item.id;

        tracing::info!(
            "Enqueued command {} from agent {} with risk level {:?}",
            id,
            item.agent_id,
            item.risk_level
        );

        // Set initial stage based on risk level
        item.stage.set(PipelineStage::Validated);

        self.items.update(|items| items.push(item));

        // Process auto-approvals
        self.process_auto_approvals();

        id
    }

    /// Get all items pending approval
    pub fn get_pending_items(&self) -> Vec<PipelineItem> {
        self.items
            .get()
            .into_iter()
            .filter(|item| item.is_pending_approval())
            .collect()
    }

    /// Get all items currently executing
    pub fn get_executing_items(&self) -> Vec<PipelineItem> {
        self.items
            .get()
            .into_iter()
            .filter(|item| item.is_executing())
            .collect()
    }

    /// Find an item by ID
    fn find_item(&self, id: Uuid) -> Option<PipelineItem> {
        self.items
            .get()
            .into_iter()
            .find(|item| item.id == id)
    }

    /// Approve a pending command
    pub fn approve(&self, id: Uuid) -> Result<(), String> {
        let item = self
            .find_item(id)
            .ok_or_else(|| format!("Item {} not found", id))?;

        if !item.is_pending_approval() {
            return Err(format!(
                "Item {} is not pending approval (current stage: {:?})",
                id,
                item.stage.get()
            ));
        }

        tracing::info!("Approved command {}: {}", id, item.command);
        item.stage.set(PipelineStage::Approved);

        Ok(())
    }

    /// Reject a pending command
    pub fn reject(&self, id: Uuid) -> Result<(), String> {
        let item = self
            .find_item(id)
            .ok_or_else(|| format!("Item {} not found", id))?;

        if !item.is_pending_approval() {
            return Err(format!(
                "Item {} is not pending approval (current stage: {:?})",
                id,
                item.stage.get()
            ));
        }

        tracing::info!("Rejected command {}: {}", id, item.command);
        item.stage.set(PipelineStage::Cancelled);

        Ok(())
    }

    /// Approve a command with modifications
    pub fn approve_modified(&self, id: Uuid, new_command: &str) -> Result<(), String> {
        let item = self
            .find_item(id)
            .ok_or_else(|| format!("Item {} not found", id))?;

        if !item.is_pending_approval() {
            return Err(format!(
                "Item {} is not pending approval (current stage: {:?})",
                id,
                item.stage.get()
            ));
        }

        tracing::info!(
            "Approved modified command {}: {} -> {}",
            id,
            item.command,
            new_command
        );

        // Update the command
        self.items.update(|items| {
            if let Some(item) = items.iter_mut().find(|i| i.id == id) {
                item.command = new_command.to_string();
                item.stage.set(PipelineStage::Approved);
            }
        });

        Ok(())
    }

    /// Mark a command as executing
    pub fn mark_executing(&self, id: Uuid) -> Result<(), String> {
        let item = self
            .find_item(id)
            .ok_or_else(|| format!("Item {} not found", id))?;

        if item.stage.get() != PipelineStage::Approved {
            return Err(format!(
                "Item {} is not approved (current stage: {:?})",
                id,
                item.stage.get()
            ));
        }

        tracing::info!("Executing command {}: {}", id, item.command);
        item.stage.set(PipelineStage::Executing);

        Ok(())
    }

    /// Mark a command as completed
    pub fn mark_completed(
        &self,
        id: Uuid,
        exit_code: i32,
        output: String,
    ) -> Result<(), String> {
        let item = self
            .find_item(id)
            .ok_or_else(|| format!("Item {} not found", id))?;

        if !item.is_executing() {
            return Err(format!(
                "Item {} is not executing (current stage: {:?})",
                id,
                item.stage.get()
            ));
        }

        tracing::info!(
            "Completed command {}: {} (exit code: {})",
            id,
            item.command,
            exit_code
        );

        item.stage.set(PipelineStage::Completed);
        item.exit_code.set(Some(exit_code));
        item.output.set(Some(output));

        Ok(())
    }

    /// Mark a command as failed
    pub fn mark_failed(&self, id: Uuid, error: String) -> Result<(), String> {
        let item = self
            .find_item(id)
            .ok_or_else(|| format!("Item {} not found", id))?;

        if !item.is_executing() {
            return Err(format!(
                "Item {} is not executing (current stage: {:?})",
                id,
                item.stage.get()
            ));
        }

        tracing::error!("Failed command {}: {} - {}", id, item.command, error);

        item.stage.set(PipelineStage::Failed);
        item.output.set(Some(error));

        Ok(())
    }

    /// Process auto-approvals based on risk level
    pub fn process_auto_approvals(&self) {
        let auto_approve_level = self.auto_approve_level.get();

        self.items.update(|items| {
            for item in items.iter_mut() {
                // Only auto-approve items that are validated but not yet approved
                if item.stage.get() == PipelineStage::Validated
                    && item.risk_level <= auto_approve_level
                {
                    tracing::info!(
                        "Auto-approved command {} (risk: {:?} <= {:?}): {}",
                        item.id,
                        item.risk_level,
                        auto_approve_level,
                        item.command
                    );
                    item.stage.set(PipelineStage::Approved);
                } else if item.stage.get() == PipelineStage::Validated {
                    // Requires manual approval
                    tracing::info!(
                        "Command {} requires manual approval (risk: {:?} > {:?}): {}",
                        item.id,
                        item.risk_level,
                        auto_approve_level,
                        item.command
                    );
                    item.stage.set(PipelineStage::PendingApproval);
                }
            }
        });
    }

    /// Set the auto-approve level
    pub fn set_auto_approve_level(&self, level: RiskLevel) {
        tracing::info!("Setting auto-approve level to {:?}", level);
        self.auto_approve_level.set(level);
        self.process_auto_approvals();
    }

    /// Get the current auto-approve level
    pub fn get_auto_approve_level(&self) -> RiskLevel {
        self.auto_approve_level.get()
    }

    /// Clear completed items from the pipeline
    pub fn clear_completed(&self) {
        self.items.update(|items| {
            items.retain(|item| !item.is_terminal());
        });
        tracing::info!("Cleared completed items from pipeline");
    }

    /// Get total count of items
    pub fn item_count(&self) -> usize {
        self.items.get().len()
    }

    /// Get count of pending items
    pub fn pending_count(&self) -> usize {
        self.items
            .get()
            .iter()
            .filter(|item| item.is_pending_approval())
            .count()
    }
}

impl Default for ExecutionPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        let pipeline = ExecutionPipeline::new();
        assert_eq!(pipeline.item_count(), 0);
        assert_eq!(pipeline.get_auto_approve_level(), RiskLevel::Low);
    }

    #[test]
    fn test_enqueue_low_risk() {
        let pipeline = ExecutionPipeline::new();
        let id = pipeline.enqueue(
            "test_agent".to_string(),
            "ls -la".to_string(),
            Some("List files".to_string()),
            RiskLevel::Low,
        );

        assert_eq!(pipeline.item_count(), 1);
        let item = pipeline.find_item(id).unwrap();
        // Low risk should be auto-approved
        assert_eq!(item.stage.get(), PipelineStage::Approved);
    }

    #[test]
    fn test_enqueue_high_risk() {
        let pipeline = ExecutionPipeline::new();
        let id = pipeline.enqueue(
            "test_agent".to_string(),
            "rm -rf /tmp/test".to_string(),
            Some("Remove directory".to_string()),
            RiskLevel::High,
        );

        assert_eq!(pipeline.item_count(), 1);
        let item = pipeline.find_item(id).unwrap();
        // High risk should require approval
        assert_eq!(item.stage.get(), PipelineStage::PendingApproval);
    }

    #[test]
    fn test_approve_command() {
        let pipeline = ExecutionPipeline::new();
        let id = pipeline.enqueue(
            "test_agent".to_string(),
            "rm file.txt".to_string(),
            None,
            RiskLevel::High,
        );

        // Should be pending approval
        assert_eq!(pipeline.pending_count(), 1);

        // Approve it
        pipeline.approve(id).unwrap();

        let item = pipeline.find_item(id).unwrap();
        assert_eq!(item.stage.get(), PipelineStage::Approved);
        assert_eq!(pipeline.pending_count(), 0);
    }

    #[test]
    fn test_reject_command() {
        let pipeline = ExecutionPipeline::new();
        let id = pipeline.enqueue(
            "test_agent".to_string(),
            "rm -rf /".to_string(),
            None,
            RiskLevel::Critical,
        );

        // Reject it
        pipeline.reject(id).unwrap();

        let item = pipeline.find_item(id).unwrap();
        assert_eq!(item.stage.get(), PipelineStage::Cancelled);
    }

    #[test]
    fn test_approve_modified() {
        let pipeline = ExecutionPipeline::new();
        let id = pipeline.enqueue(
            "test_agent".to_string(),
            "rm -rf /tmp/dangerous".to_string(),
            None,
            RiskLevel::Critical,
        );

        // Modify and approve
        pipeline
            .approve_modified(id, "rm -rf /tmp/safe")
            .unwrap();

        let item = pipeline.find_item(id).unwrap();
        assert_eq!(item.stage.get(), PipelineStage::Approved);
        assert_eq!(item.command, "rm -rf /tmp/safe");
    }

    #[test]
    fn test_execution_lifecycle() {
        let pipeline = ExecutionPipeline::new();
        let id = pipeline.enqueue(
            "test_agent".to_string(),
            "echo hello".to_string(),
            None,
            RiskLevel::Low,
        );

        // Should be auto-approved
        let item = pipeline.find_item(id).unwrap();
        assert_eq!(item.stage.get(), PipelineStage::Approved);

        // Mark as executing
        pipeline.mark_executing(id).unwrap();
        let item = pipeline.find_item(id).unwrap();
        assert_eq!(item.stage.get(), PipelineStage::Executing);

        // Mark as completed
        pipeline
            .mark_completed(id, 0, "hello\n".to_string())
            .unwrap();
        let item = pipeline.find_item(id).unwrap();
        assert_eq!(item.stage.get(), PipelineStage::Completed);
        assert_eq!(item.exit_code.get(), Some(0));
        assert_eq!(item.output.get(), Some("hello\n".to_string()));
    }

    #[test]
    fn test_mark_failed() {
        let pipeline = ExecutionPipeline::new();
        let id = pipeline.enqueue(
            "test_agent".to_string(),
            "false".to_string(),
            None,
            RiskLevel::Low,
        );

        // Mark as executing then failed
        pipeline.mark_executing(id).unwrap();
        pipeline
            .mark_failed(id, "Command failed with exit code 1".to_string())
            .unwrap();

        let item = pipeline.find_item(id).unwrap();
        assert_eq!(item.stage.get(), PipelineStage::Failed);
        assert!(item
            .output
            .get()
            .unwrap()
            .contains("Command failed"));
    }

    #[test]
    fn test_auto_approve_level() {
        let pipeline = ExecutionPipeline::new();

        // Set to auto-approve medium risk
        pipeline.set_auto_approve_level(RiskLevel::Medium);

        let id1 = pipeline.enqueue(
            "test_agent".to_string(),
            "ls".to_string(),
            None,
            RiskLevel::Low,
        );
        let id2 = pipeline.enqueue(
            "test_agent".to_string(),
            "mkdir test".to_string(),
            None,
            RiskLevel::Medium,
        );
        let id3 = pipeline.enqueue(
            "test_agent".to_string(),
            "rm file".to_string(),
            None,
            RiskLevel::High,
        );

        // Low and medium should be auto-approved
        assert_eq!(
            pipeline.find_item(id1).unwrap().stage.get(),
            PipelineStage::Approved
        );
        assert_eq!(
            pipeline.find_item(id2).unwrap().stage.get(),
            PipelineStage::Approved
        );
        // High should require approval
        assert_eq!(
            pipeline.find_item(id3).unwrap().stage.get(),
            PipelineStage::PendingApproval
        );
    }

    #[test]
    fn test_clear_completed() {
        let pipeline = ExecutionPipeline::new();

        let id1 = pipeline.enqueue(
            "test_agent".to_string(),
            "echo 1".to_string(),
            None,
            RiskLevel::Low,
        );
        let id2 = pipeline.enqueue(
            "test_agent".to_string(),
            "echo 2".to_string(),
            None,
            RiskLevel::Low,
        );

        // Complete first one
        pipeline.mark_executing(id1).unwrap();
        pipeline.mark_completed(id1, 0, "1\n".to_string()).unwrap();

        assert_eq!(pipeline.item_count(), 2);

        // Clear completed
        pipeline.clear_completed();

        assert_eq!(pipeline.item_count(), 1);
        assert!(pipeline.find_item(id1).is_none());
        assert!(pipeline.find_item(id2).is_some());
    }

    #[test]
    fn test_get_pending_items() {
        let pipeline = ExecutionPipeline::new();

        pipeline.enqueue(
            "test_agent".to_string(),
            "ls".to_string(),
            None,
            RiskLevel::Low,
        );
        pipeline.enqueue(
            "test_agent".to_string(),
            "rm file".to_string(),
            None,
            RiskLevel::High,
        );
        pipeline.enqueue(
            "test_agent".to_string(),
            "rm -rf /".to_string(),
            None,
            RiskLevel::Critical,
        );

        let pending = pipeline.get_pending_items();
        assert_eq!(pending.len(), 2); // High and Critical risk
    }

    #[test]
    fn test_pipeline_item_is_terminal() {
        let item = PipelineItem::new(
            "test".to_string(),
            "test".to_string(),
            None,
            RiskLevel::Low,
        );

        item.stage.set(PipelineStage::Executing);
        assert!(!item.is_terminal());

        item.stage.set(PipelineStage::Completed);
        assert!(item.is_terminal());

        item.stage.set(PipelineStage::Failed);
        assert!(item.is_terminal());

        item.stage.set(PipelineStage::Cancelled);
        assert!(item.is_terminal());
    }
}

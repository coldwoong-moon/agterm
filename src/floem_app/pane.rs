//! Pane Management (tmux-style splitting)
//!
//! Recursive pane tree structure for horizontal/vertical splits.

use floem::reactive::{RwSignal, SignalWith};
use std::sync::Arc;
use uuid::Uuid;

use crate::floem_app::views::terminal::TerminalState;
use crate::terminal::pty::PtyManager;

/// Split direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

/// Pane tree node - recursive structure for splits
#[derive(Clone)]
pub enum PaneTree {
    /// Split node - contains two children with a divider ratio
    Split {
        direction: SplitDirection,
        /// Left or top child
        first: Arc<RwSignal<PaneTree>>,
        /// Right or bottom child
        second: Arc<RwSignal<PaneTree>>,
        /// Split ratio (0.0 to 1.0, represents first child's proportion)
        ratio: RwSignal<f64>,
    },
    /// Leaf node - contains a terminal
    Leaf {
        id: Uuid,
        terminal_state: TerminalState,
        /// Whether this pane has focus
        is_focused: RwSignal<bool>,
    },
}

impl PaneTree {
    /// Create a new leaf pane with a terminal
    pub fn new_leaf(pty_manager: &Arc<PtyManager>) -> Self {
        let pane_id = Uuid::new_v4();
        let terminal_state = TerminalState::new();

        tracing::debug!("Creating new pane with ID: {}", pane_id);

        // Create PTY session for this pane with retry logic
        const MAX_RETRIES: u32 = 3;
        const RETRY_DELAY_MS: u64 = 100;

        let mut session_created = false;
        for attempt in 1..=MAX_RETRIES {
            match pty_manager.create_session(24, 80) {
                Ok(session_id) => {
                    terminal_state.set_pty_session(session_id);
                    tracing::info!(
                        "Created PTY session {} for pane {} (attempt {})",
                        session_id,
                        pane_id,
                        attempt
                    );

                    // Start PTY output polling thread with ADAPTIVE POLLING
                    let terminal_state_clone = terminal_state.clone();
                    let pty_manager_clone = pty_manager.clone();
                    let pane_id_clone = pane_id;
                    std::thread::spawn(move || {
                        tracing::debug!("Starting adaptive PTY polling thread for pane {}", pane_id_clone);

                        // Adaptive polling parameters
                        let mut poll_interval_ms = 8;  // Start fast (8ms = 125 Hz)
                        const MIN_POLL_MS: u64 = 8;    // Min 8ms (fastest)
                        const MAX_POLL_MS: u64 = 50;   // Max 50ms when idle
                        let mut idle_count = 0;

                        loop {
                            std::thread::sleep(std::time::Duration::from_millis(poll_interval_ms));

                            if let Some(session_id) = terminal_state_clone.pty_session() {
                                match pty_manager_clone.read(&session_id) {
                                    Ok(data) if !data.is_empty() => {
                                        // Data received - batch process it
                                        tracing::info!("PTY output received: {} bytes", data.len());
                                        terminal_state_clone.process_output(&data);

                                        // Reset to fast polling when activity detected
                                        poll_interval_ms = MIN_POLL_MS;
                                        idle_count = 0;

                                        // Try to read more data immediately (batch processing)
                                        for _ in 0..10 {  // Read up to 10 more chunks
                                            match pty_manager_clone.read(&session_id) {
                                                Ok(more_data) if !more_data.is_empty() => {
                                                    terminal_state_clone.process_output(&more_data);
                                                }
                                                _ => break,
                                            }
                                        }
                                    }
                                    Ok(_) => {
                                        // No data - increase polling interval (adaptive backoff)
                                        idle_count += 1;
                                        if idle_count > 10 && poll_interval_ms < MAX_POLL_MS {
                                            poll_interval_ms = (poll_interval_ms + 2).min(MAX_POLL_MS);
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!(
                                            "Error reading from PTY session {} for pane {}: {}",
                                            session_id,
                                            pane_id_clone,
                                            e
                                        );
                                        break;
                                    }
                                }
                            } else {
                                tracing::debug!("PTY session closed for pane {}", pane_id_clone);
                                break;
                            }
                        }
                        tracing::info!("PTY polling thread terminated for pane {}", pane_id_clone);
                    });

                    session_created = true;
                    break;
                }
                Err(e) => {
                    if attempt < MAX_RETRIES {
                        tracing::warn!(
                            "Failed to create PTY session for pane {} (attempt {}): {}. Retrying...",
                            pane_id,
                            attempt,
                            e
                        );
                        std::thread::sleep(std::time::Duration::from_millis(RETRY_DELAY_MS));
                    } else {
                        tracing::error!(
                            "Failed to create PTY session for pane {} after {} attempts: {}",
                            pane_id,
                            MAX_RETRIES,
                            e
                        );
                    }
                }
            }
        }

        if !session_created {
            tracing::error!(
                "Pane {} created without PTY session - terminal will not be functional",
                pane_id
            );
        }

        Self::Leaf {
            id: pane_id,
            terminal_state,
            is_focused: RwSignal::new(true),  // New panes are focused by default
        }
    }

    /// Split this pane horizontally (left | right)
    pub fn split_horizontal(&mut self, pty_manager: &Arc<PtyManager>) {
        tracing::info!("Splitting pane horizontally");
        let current = self.clone();
        let new_pane = Self::new_leaf(pty_manager);

        *self = Self::Split {
            direction: SplitDirection::Horizontal,
            first: Arc::new(RwSignal::new(current)),
            second: Arc::new(RwSignal::new(new_pane)),
            ratio: RwSignal::new(0.5),
        };
        tracing::debug!("Horizontal split completed");
    }

    /// Split this pane vertically (top / bottom)
    pub fn split_vertical(&mut self, pty_manager: &Arc<PtyManager>) {
        tracing::info!("Splitting pane vertically");
        let current = self.clone();
        let new_pane = Self::new_leaf(pty_manager);

        *self = Self::Split {
            direction: SplitDirection::Vertical,
            first: Arc::new(RwSignal::new(current)),
            second: Arc::new(RwSignal::new(new_pane)),
            ratio: RwSignal::new(0.5),
        };
        tracing::debug!("Vertical split completed");
    }

    /// Get the focused leaf pane (recursive search)
    pub fn get_focused_leaf(&self) -> Option<(Uuid, TerminalState)> {
        use floem::reactive::SignalGet;

        match self {
            Self::Leaf { id, terminal_state, is_focused } => {
                if is_focused.get() {
                    Some((*id, terminal_state.clone()))
                } else {
                    None
                }
            }
            Self::Split { first, second, .. } => {
                first.with(|t| t.get_focused_leaf())
                    .or_else(|| second.with(|t| t.get_focused_leaf()))
            }
        }
    }

    /// Get the title from the focused leaf pane (future feature)
    #[allow(dead_code)]
    pub fn get_focused_title(&self, default: &str) -> String {
        if let Some((_, terminal_state)) = self.get_focused_leaf() {
            terminal_state.window_title()
                .unwrap_or_else(|| default.to_string())
        } else {
            default.to_string()
        }
    }

    /// Set focus on a specific pane by ID
    pub fn set_focus(&self, pane_id: Uuid) -> bool {
        use floem::reactive::SignalUpdate;

        match self {
            Self::Leaf { id, is_focused, .. } => {
                let is_match = *id == pane_id;
                is_focused.set(is_match);
                is_match
            }
            Self::Split { first, second, .. } => {
                let found_first = first.with(|t| t.set_focus(pane_id));
                let found_second = second.with(|t| t.set_focus(pane_id));
                found_first || found_second
            }
        }
    }

    /// Clear focus from all panes
    pub fn clear_focus(&self) {
        use floem::reactive::SignalUpdate;

        match self {
            Self::Leaf { is_focused, .. } => {
                is_focused.set(false);
            }
            Self::Split { first, second, .. } => {
                first.with(|t| t.clear_focus());
                second.with(|t| t.clear_focus());
            }
        }
    }

    /// Get all leaf pane IDs (for navigation)
    pub fn get_all_leaf_ids(&self) -> Vec<Uuid> {
        match self {
            Self::Leaf { id, .. } => vec![*id],
            Self::Split { first, second, .. } => {
                let mut ids = first.with(|t| t.get_all_leaf_ids());
                ids.extend(second.with(|t| t.get_all_leaf_ids()));
                ids
            }
        }
    }

    /// Navigate to the next pane in the given direction
    pub fn navigate(&self, direction: NavigationDirection) -> Option<Uuid> {
        let all_ids = self.get_all_leaf_ids();
        let current_focused = self.get_focused_leaf().map(|(id, _)| id);

        if let Some(current_id) = current_focused {
            if let Some(current_idx) = all_ids.iter().position(|id| *id == current_id) {
                let next_idx = match direction {
                    NavigationDirection::Next => (current_idx + 1) % all_ids.len(),
                    NavigationDirection::Previous => {
                        if current_idx == 0 {
                            all_ids.len() - 1
                        } else {
                            current_idx - 1
                        }
                    }
                };
                return Some(all_ids[next_idx]);
            }
        }

        // Fallback: return first pane
        all_ids.first().copied()
    }

    /// Close the focused pane (future feature)
    #[allow(dead_code)]
    pub fn close_focused_pane(&mut self, pty_manager: &Arc<PtyManager>) -> bool {
        // Get focused pane ID
        let focused_id = match self.get_focused_leaf() {
            Some((id, _)) => {
                tracing::info!("Closing focused pane with ID: {}", id);
                id
            }
            None => {
                tracing::warn!("No focused pane to close");
                return false;
            }
        };

        // Cleanup PTY session
        if let Some((_, terminal_state)) = self.get_focused_leaf() {
            if let Some(session_id) = terminal_state.pty_session() {
                tracing::debug!("Cleaning up PTY session {} for pane {}", session_id, focused_id);
                if let Err(e) = pty_manager.close_session(&session_id) {
                    tracing::error!("Failed to close PTY session {} for pane {}: {}", session_id, focused_id, e);
                } else {
                    tracing::info!("Closed PTY session {} for pane {}", session_id, focused_id);
                }
            } else {
                tracing::debug!("Pane {} has no active PTY session", focused_id);
            }
        }

        // Remove the pane from the tree
        let removed = self.remove_pane(focused_id);
        if removed {
            tracing::info!("Pane {} successfully removed from tree", focused_id);
        } else {
            tracing::warn!("Failed to remove pane {} from tree", focused_id);
        }
        removed
    }

    /// Remove a pane by ID (internal helper, future feature)
    #[allow(dead_code)]
    fn remove_pane(&mut self, pane_id: Uuid) -> bool {
        use floem::reactive::{SignalGet, SignalUpdate};

        match self {
            Self::Leaf { id, .. } => *id == pane_id,
            Self::Split { first, second, .. } => {
                // Check if either child is the target pane
                let first_val = first.get();
                let second_val = second.get();

                match (&first_val, &second_val) {
                    (Self::Leaf { id, .. }, _) if *id == pane_id => {
                        // Replace this split with the second child
                        *self = second_val.clone();
                        true
                    }
                    (_, Self::Leaf { id, .. }) if *id == pane_id => {
                        // Replace this split with the first child
                        *self = first_val.clone();
                        true
                    }
                    _ => {
                        // Recursively search children
                        let mut first_mut = first.get();
                        if first_mut.remove_pane(pane_id) {
                            first.set(first_mut);
                            return true;
                        }

                        let mut second_mut = second.get();
                        if second_mut.remove_pane(pane_id) {
                            second.set(second_mut);
                            return true;
                        }

                        false
                    }
                }
            }
        }
    }

    /// Count total number of leaf panes
    pub fn count_leaves(&self) -> usize {
        match self {
            Self::Leaf { .. } => 1,
            Self::Split { first, second, .. } => {
                first.with(|t| t.count_leaves()) + second.with(|t| t.count_leaves())
            }
        }
    }
}

/// Pane navigation direction
#[derive(Debug, Clone, Copy)]
pub enum NavigationDirection {
    Next,
    Previous,
}

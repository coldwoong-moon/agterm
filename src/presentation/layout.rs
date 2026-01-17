//! Split Layout System
//!
//! Tree-based layout system for split terminal panes.
//! Inspired by tmux/zellij layout models.

use crate::infrastructure::pty::PtyId;
use ratatui::layout::Rect;
use std::collections::HashMap;
use uuid::Uuid;

/// Layout node identifier
pub type NodeId = Uuid;

/// Split direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    /// Split horizontally (side by side: left | right)
    Horizontal,
    /// Split vertically (stacked: top / bottom)
    Vertical,
}

/// A node in the layout tree
#[derive(Debug, Clone)]
pub enum LayoutNode {
    /// A leaf node containing a terminal
    Terminal {
        /// Node ID
        id: NodeId,
        /// Associated PTY session ID
        pty_id: PtyId,
    },
    /// A split node containing two children
    Split {
        /// Node ID
        id: NodeId,
        /// Split direction
        direction: SplitDirection,
        /// First child (left/top)
        first: Box<LayoutNode>,
        /// Second child (right/bottom)
        second: Box<LayoutNode>,
        /// Split ratio (0.0 - 1.0, proportion of first child)
        ratio: f32,
    },
}

impl LayoutNode {
    /// Create a new terminal node
    pub fn terminal(pty_id: PtyId) -> Self {
        LayoutNode::Terminal {
            id: Uuid::new_v4(),
            pty_id,
        }
    }

    /// Create a new split node
    pub fn split(direction: SplitDirection, first: LayoutNode, second: LayoutNode) -> Self {
        LayoutNode::Split {
            id: Uuid::new_v4(),
            direction,
            first: Box::new(first),
            second: Box::new(second),
            ratio: 0.5,
        }
    }

    /// Get the node ID
    pub fn id(&self) -> NodeId {
        match self {
            LayoutNode::Terminal { id, .. } => *id,
            LayoutNode::Split { id, .. } => *id,
        }
    }

    /// Check if this is a terminal node
    pub fn is_terminal(&self) -> bool {
        matches!(self, LayoutNode::Terminal { .. })
    }

    /// Get PTY ID if this is a terminal node
    pub fn pty_id(&self) -> Option<PtyId> {
        match self {
            LayoutNode::Terminal { pty_id, .. } => Some(*pty_id),
            LayoutNode::Split { .. } => None,
        }
    }

    /// Count terminal nodes
    pub fn terminal_count(&self) -> usize {
        match self {
            LayoutNode::Terminal { .. } => 1,
            LayoutNode::Split { first, second, .. } => {
                first.terminal_count() + second.terminal_count()
            }
        }
    }

    /// Get all PTY IDs in this subtree
    pub fn all_pty_ids(&self) -> Vec<PtyId> {
        match self {
            LayoutNode::Terminal { pty_id, .. } => vec![*pty_id],
            LayoutNode::Split { first, second, .. } => {
                let mut ids = first.all_pty_ids();
                ids.extend(second.all_pty_ids());
                ids
            }
        }
    }

    /// Find a node by its ID
    pub fn find_node(&self, node_id: &NodeId) -> Option<&LayoutNode> {
        if &self.id() == node_id {
            return Some(self);
        }

        match self {
            LayoutNode::Terminal { .. } => None,
            LayoutNode::Split { first, second, .. } => first
                .find_node(node_id)
                .or_else(|| second.find_node(node_id)),
        }
    }

    /// Find a node by PTY ID
    pub fn find_by_pty(&self, pty_id: &PtyId) -> Option<&LayoutNode> {
        match self {
            LayoutNode::Terminal {
                pty_id: pid,
                id: _,
            } if pid == pty_id => Some(self),
            LayoutNode::Terminal { .. } => None,
            LayoutNode::Split { first, second, .. } => first
                .find_by_pty(pty_id)
                .or_else(|| second.find_by_pty(pty_id)),
        }
    }

    /// Find parent of a node
    pub fn find_parent(&self, node_id: &NodeId) -> Option<&LayoutNode> {
        match self {
            LayoutNode::Terminal { .. } => None,
            LayoutNode::Split { first, second, .. } => {
                if first.id() == *node_id || second.id() == *node_id {
                    return Some(self);
                }
                first
                    .find_parent(node_id)
                    .or_else(|| second.find_parent(node_id))
            }
        }
    }

    /// Replace a terminal node with a split
    pub fn split_terminal(
        &mut self,
        pty_id: &PtyId,
        direction: SplitDirection,
        new_pty_id: PtyId,
        new_first: bool,
    ) -> bool {
        match self {
            LayoutNode::Terminal {
                pty_id: pid,
                id: _,
            } if pid == pty_id => {
                let existing = LayoutNode::terminal(*pid);
                let new_terminal = LayoutNode::terminal(new_pty_id);

                let (first, second) = if new_first {
                    (new_terminal, existing)
                } else {
                    (existing, new_terminal)
                };

                *self = LayoutNode::split(direction, first, second);
                true
            }
            LayoutNode::Terminal { .. } => false,
            LayoutNode::Split { first, second, .. } => {
                first.split_terminal(pty_id, direction, new_pty_id, new_first)
                    || second.split_terminal(pty_id, direction, new_pty_id, new_first)
            }
        }
    }

    /// Remove a terminal and collapse the split
    /// Returns the remaining node if removal was successful
    pub fn remove_terminal(&mut self, pty_id: &PtyId) -> Option<LayoutNode> {
        match self {
            LayoutNode::Terminal { .. } => None, // Cannot remove from single terminal
            LayoutNode::Split { first, second, .. } => {
                // Check if first child is the target
                if let LayoutNode::Terminal {
                    pty_id: pid,
                    id: _,
                } = first.as_ref()
                {
                    if pid == pty_id {
                        return Some(second.as_ref().clone());
                    }
                }

                // Check if second child is the target
                if let LayoutNode::Terminal {
                    pty_id: pid,
                    id: _,
                } = second.as_ref()
                {
                    if pid == pty_id {
                        return Some(first.as_ref().clone());
                    }
                }

                // Recursively try to remove from children
                if let Some(remaining) = first.remove_terminal(pty_id) {
                    *first = Box::new(remaining);
                    return None;
                }
                if let Some(remaining) = second.remove_terminal(pty_id) {
                    *second = Box::new(remaining);
                    return None;
                }

                None
            }
        }
    }

    /// Adjust split ratio
    pub fn adjust_ratio(&mut self, node_id: &NodeId, delta: f32) -> bool {
        match self {
            LayoutNode::Terminal { .. } => false,
            LayoutNode::Split {
                id,
                ratio,
                first,
                second,
                ..
            } => {
                if id == node_id {
                    *ratio = (*ratio + delta).clamp(0.1, 0.9);
                    return true;
                }
                first.adjust_ratio(node_id, delta) || second.adjust_ratio(node_id, delta)
            }
        }
    }
}

/// Computed layout with absolute positions
#[derive(Debug, Clone)]
pub struct ComputedLayout {
    /// Map from node ID to computed rect
    pub rects: HashMap<NodeId, Rect>,
    /// Map from PTY ID to computed rect
    pub pty_rects: HashMap<PtyId, Rect>,
}

impl ComputedLayout {
    /// Create a new empty computed layout
    pub fn new() -> Self {
        Self {
            rects: HashMap::new(),
            pty_rects: HashMap::new(),
        }
    }

    /// Compute layout for a node tree within a given area
    pub fn compute(node: &LayoutNode, area: Rect) -> Self {
        let mut layout = Self::new();
        layout.compute_recursive(node, area);
        layout
    }

    fn compute_recursive(&mut self, node: &LayoutNode, area: Rect) {
        self.rects.insert(node.id(), area);

        match node {
            LayoutNode::Terminal { pty_id, .. } => {
                self.pty_rects.insert(*pty_id, area);
            }
            LayoutNode::Split {
                direction,
                first,
                second,
                ratio,
                ..
            } => {
                let (first_area, second_area) = match direction {
                    SplitDirection::Horizontal => {
                        let first_width = (area.width as f32 * ratio) as u16;
                        let second_width = area.width.saturating_sub(first_width).saturating_sub(1); // -1 for separator

                        let first_rect = Rect {
                            x: area.x,
                            y: area.y,
                            width: first_width,
                            height: area.height,
                        };
                        let second_rect = Rect {
                            x: area.x + first_width + 1, // +1 for separator
                            y: area.y,
                            width: second_width,
                            height: area.height,
                        };
                        (first_rect, second_rect)
                    }
                    SplitDirection::Vertical => {
                        let first_height = (area.height as f32 * ratio) as u16;
                        let second_height =
                            area.height.saturating_sub(first_height).saturating_sub(1); // -1 for separator

                        let first_rect = Rect {
                            x: area.x,
                            y: area.y,
                            width: area.width,
                            height: first_height,
                        };
                        let second_rect = Rect {
                            x: area.x,
                            y: area.y + first_height + 1, // +1 for separator
                            width: area.width,
                            height: second_height,
                        };
                        (first_rect, second_rect)
                    }
                };

                self.compute_recursive(first, first_area);
                self.compute_recursive(second, second_area);
            }
        }
    }

    /// Get rect for a PTY
    pub fn get_pty_rect(&self, pty_id: &PtyId) -> Option<&Rect> {
        self.pty_rects.get(pty_id)
    }
}

impl Default for ComputedLayout {
    fn default() -> Self {
        Self::new()
    }
}

/// Layout manager that maintains the layout tree and focus state
#[derive(Debug)]
pub struct LayoutManager {
    /// Root node of the layout tree
    root: Option<LayoutNode>,
    /// Currently focused PTY ID
    focused_pty: Option<PtyId>,
    /// Cached computed layout
    cached_layout: Option<ComputedLayout>,
    /// Area used for cached layout
    cached_area: Option<Rect>,
}

impl LayoutManager {
    /// Create a new layout manager
    pub fn new() -> Self {
        Self {
            root: None,
            focused_pty: None,
            cached_layout: None,
            cached_area: None,
        }
    }

    /// Check if the layout is empty
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// Get the root node
    pub fn root(&self) -> Option<&LayoutNode> {
        self.root.as_ref()
    }

    /// Get the focused PTY ID
    pub fn focused_pty(&self) -> Option<PtyId> {
        self.focused_pty
    }

    /// Set the focused PTY
    pub fn set_focus(&mut self, pty_id: PtyId) {
        if let Some(ref root) = self.root {
            if root.find_by_pty(&pty_id).is_some() {
                self.focused_pty = Some(pty_id);
            }
        }
    }

    /// Add a terminal to the layout
    /// If empty, becomes the root. Otherwise, splits the focused terminal.
    pub fn add_terminal(
        &mut self,
        pty_id: PtyId,
        direction: Option<SplitDirection>,
        _new_first: bool,
    ) {
        self.invalidate_cache();

        match &mut self.root {
            None => {
                // First terminal becomes root
                self.root = Some(LayoutNode::terminal(pty_id));
                self.focused_pty = Some(pty_id);
            }
            Some(root) => {
                // Split the focused terminal (or first terminal if none focused)
                let target_pty = self.focused_pty.or_else(|| root.all_pty_ids().first().copied());

                if let Some(target) = target_pty {
                    let dir = direction.unwrap_or(SplitDirection::Horizontal);
                    root.split_terminal(&target, dir, pty_id, false);
                    self.focused_pty = Some(pty_id);
                }
            }
        }
    }

    /// Remove a terminal from the layout
    pub fn remove_terminal(&mut self, pty_id: &PtyId) -> bool {
        self.invalidate_cache();

        let Some(ref mut root) = self.root else {
            return false;
        };

        // Check if root is the terminal to remove
        if let LayoutNode::Terminal {
            pty_id: pid,
            id: _,
        } = root
        {
            if pid == pty_id {
                self.root = None;
                self.focused_pty = None;
                return true;
            }
            return false;
        }

        // Try to remove from tree
        if let Some(remaining) = root.remove_terminal(pty_id) {
            self.root = Some(remaining);

            // Update focus if removed terminal was focused
            if self.focused_pty == Some(*pty_id) {
                self.focused_pty = self.root.as_ref().and_then(|r| r.all_pty_ids().first().copied());
            }
            return true;
        }

        false
    }

    /// Get all PTY IDs in the layout
    pub fn all_pty_ids(&self) -> Vec<PtyId> {
        self.root
            .as_ref()
            .map(|r| r.all_pty_ids())
            .unwrap_or_default()
    }

    /// Compute layout for a given area
    pub fn compute(&mut self, area: Rect) -> &ComputedLayout {
        // Check if cache is valid
        let needs_recompute = match (&self.cached_layout, &self.cached_area) {
            (Some(_), Some(cached_area)) if cached_area == &area => false,
            _ => true,
        };

        if needs_recompute {
            // Recompute
            let computed = self
                .root
                .as_ref()
                .map(|root| ComputedLayout::compute(root, area))
                .unwrap_or_default();

            self.cached_layout = Some(computed);
            self.cached_area = Some(area);
        }

        self.cached_layout.as_ref().unwrap()
    }

    /// Invalidate the cached layout
    pub fn invalidate_cache(&mut self) {
        self.cached_layout = None;
        self.cached_area = None;
    }

    /// Navigate focus in a direction (vim-style)
    pub fn navigate(&mut self, direction: NavigateDirection) -> Option<PtyId> {
        if self.root.is_none() {
            return None;
        }

        let Some(current_pty) = self.focused_pty else {
            return None;
        };

        let Some(ref cached) = self.cached_layout else {
            return None;
        };

        let current_rect = cached.get_pty_rect(&current_pty)?;

        // Find the best candidate in the given direction
        let candidates: Vec<_> = cached
            .pty_rects
            .iter()
            .filter(|(id, _)| **id != current_pty)
            .filter(|(_, rect)| {
                match direction {
                    NavigateDirection::Left => {
                        rect.x + rect.width <= current_rect.x
                            && rects_overlap_vertically(rect, current_rect)
                    }
                    NavigateDirection::Right => {
                        rect.x >= current_rect.x + current_rect.width
                            && rects_overlap_vertically(rect, current_rect)
                    }
                    NavigateDirection::Up => {
                        rect.y + rect.height <= current_rect.y
                            && rects_overlap_horizontally(rect, current_rect)
                    }
                    NavigateDirection::Down => {
                        rect.y >= current_rect.y + current_rect.height
                            && rects_overlap_horizontally(rect, current_rect)
                    }
                }
            })
            .collect();

        // Find closest candidate
        let best = candidates.iter().min_by_key(|(_, rect)| {
            match direction {
                NavigateDirection::Left => current_rect.x.saturating_sub(rect.x + rect.width),
                NavigateDirection::Right => rect.x.saturating_sub(current_rect.x + current_rect.width),
                NavigateDirection::Up => current_rect.y.saturating_sub(rect.y + rect.height),
                NavigateDirection::Down => rect.y.saturating_sub(current_rect.y + current_rect.height),
            }
        });

        if let Some((pty_id, _)) = best {
            self.focused_pty = Some(**pty_id);
            Some(**pty_id)
        } else {
            None
        }
    }

    /// Cycle focus to next terminal
    pub fn focus_next(&mut self) -> Option<PtyId> {
        let pty_ids = self.all_pty_ids();
        if pty_ids.is_empty() {
            return None;
        }

        let current_idx = self
            .focused_pty
            .and_then(|id| pty_ids.iter().position(|pid| *pid == id))
            .unwrap_or(0);

        let next_idx = (current_idx + 1) % pty_ids.len();
        let next_pty = pty_ids[next_idx];
        self.focused_pty = Some(next_pty);
        Some(next_pty)
    }

    /// Cycle focus to previous terminal
    pub fn focus_prev(&mut self) -> Option<PtyId> {
        let pty_ids = self.all_pty_ids();
        if pty_ids.is_empty() {
            return None;
        }

        let current_idx = self
            .focused_pty
            .and_then(|id| pty_ids.iter().position(|pid| *pid == id))
            .unwrap_or(0);

        let prev_idx = if current_idx == 0 {
            pty_ids.len() - 1
        } else {
            current_idx - 1
        };
        let prev_pty = pty_ids[prev_idx];
        self.focused_pty = Some(prev_pty);
        Some(prev_pty)
    }

    /// Get terminal count
    pub fn terminal_count(&self) -> usize {
        self.root.as_ref().map(|r| r.terminal_count()).unwrap_or(0)
    }

    /// Resize the split containing the focused terminal
    pub fn resize_focused(&mut self, delta: f32) -> bool {
        self.invalidate_cache();

        let Some(ref mut root) = self.root else {
            return false;
        };

        let Some(focused) = self.focused_pty else {
            return false;
        };

        // Find the node containing this PTY
        let node = root.find_by_pty(&focused);
        if let Some(node) = node {
            // Find parent split and adjust its ratio
            if let Some(parent) = root.find_parent(&node.id()) {
                return root.adjust_ratio(&parent.id(), delta);
            }
        }

        false
    }
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Navigation direction for vim-style movement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigateDirection {
    /// Move left (h)
    Left,
    /// Move down (j)
    Down,
    /// Move up (k)
    Up,
    /// Move right (l)
    Right,
}

/// Check if two rects overlap vertically
fn rects_overlap_vertically(a: &Rect, b: &Rect) -> bool {
    let a_top = a.y;
    let a_bottom = a.y + a.height;
    let b_top = b.y;
    let b_bottom = b.y + b.height;

    a_top < b_bottom && b_top < a_bottom
}

/// Check if two rects overlap horizontally
fn rects_overlap_horizontally(a: &Rect, b: &Rect) -> bool {
    let a_left = a.x;
    let a_right = a.x + a.width;
    let b_left = b.x;
    let b_right = b.x + b.width;

    a_left < b_right && b_left < a_right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_terminal() {
        let mut manager = LayoutManager::new();
        let pty_id = Uuid::new_v4();

        manager.add_terminal(pty_id, None, false);

        assert_eq!(manager.terminal_count(), 1);
        assert_eq!(manager.focused_pty(), Some(pty_id));
    }

    #[test]
    fn test_horizontal_split() {
        let mut manager = LayoutManager::new();
        let pty1 = Uuid::new_v4();
        let pty2 = Uuid::new_v4();

        manager.add_terminal(pty1, None, false);
        manager.add_terminal(pty2, Some(SplitDirection::Horizontal), false);

        assert_eq!(manager.terminal_count(), 2);
        assert_eq!(manager.focused_pty(), Some(pty2));
    }

    #[test]
    fn test_vertical_split() {
        let mut manager = LayoutManager::new();
        let pty1 = Uuid::new_v4();
        let pty2 = Uuid::new_v4();

        manager.add_terminal(pty1, None, false);
        manager.add_terminal(pty2, Some(SplitDirection::Vertical), false);

        assert_eq!(manager.terminal_count(), 2);
    }

    #[test]
    fn test_remove_terminal() {
        let mut manager = LayoutManager::new();
        let pty1 = Uuid::new_v4();
        let pty2 = Uuid::new_v4();

        manager.add_terminal(pty1, None, false);
        manager.add_terminal(pty2, Some(SplitDirection::Horizontal), false);

        assert!(manager.remove_terminal(&pty2));
        assert_eq!(manager.terminal_count(), 1);
        assert_eq!(manager.focused_pty(), Some(pty1));
    }

    #[test]
    fn test_focus_cycle() {
        let mut manager = LayoutManager::new();
        let pty1 = Uuid::new_v4();
        let pty2 = Uuid::new_v4();
        let pty3 = Uuid::new_v4();

        manager.add_terminal(pty1, None, false);
        manager.add_terminal(pty2, Some(SplitDirection::Horizontal), false);
        manager.add_terminal(pty3, Some(SplitDirection::Horizontal), false);

        // Focus should be on pty3 (last added)
        assert_eq!(manager.focused_pty(), Some(pty3));

        // Cycle next wraps around
        manager.focus_next();
        let focused = manager.focused_pty().unwrap();
        assert!(focused == pty1 || focused == pty2); // Depends on tree structure
    }

    #[test]
    fn test_compute_layout() {
        let mut manager = LayoutManager::new();
        let pty1 = Uuid::new_v4();
        let pty2 = Uuid::new_v4();

        manager.add_terminal(pty1, None, false);
        manager.add_terminal(pty2, Some(SplitDirection::Horizontal), false);

        let area = Rect::new(0, 0, 100, 50);
        let layout = manager.compute(area);

        assert!(layout.get_pty_rect(&pty1).is_some());
        assert!(layout.get_pty_rect(&pty2).is_some());

        // Check that rects don't overlap
        let rect1 = layout.get_pty_rect(&pty1).unwrap();
        let rect2 = layout.get_pty_rect(&pty2).unwrap();

        assert!(rect1.x + rect1.width < rect2.x || rect2.x + rect2.width < rect1.x);
    }
}

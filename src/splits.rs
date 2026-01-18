//! Terminal Split Management
//!
//! This module provides a tree-based split management system for terminal panes.
//! It supports arbitrary nesting of horizontal and vertical splits, dynamic resizing,
//! and focus navigation.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Direction of split
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitDirection {
    /// Horizontal split (divides top/bottom)
    Horizontal,
    /// Vertical split (divides left/right)
    Vertical,
}

impl fmt::Display for SplitDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SplitDirection::Horizontal => write!(f, "Horizontal"),
            SplitDirection::Vertical => write!(f, "Vertical"),
        }
    }
}

/// Direction for focus navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationDirection {
    Up,
    Down,
    Left,
    Right,
}

/// A node in the split tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SplitNode {
    /// A leaf node containing a terminal pane
    Leaf {
        /// Unique pane ID
        id: usize,
        /// Size ratio (0.0 - 1.0) relative to parent
        ratio: f32,
    },
    /// A split node containing two child nodes
    Split {
        /// Split direction
        direction: SplitDirection,
        /// First child (left or top)
        first: Box<SplitNode>,
        /// Second child (right or bottom)
        second: Box<SplitNode>,
        /// Split ratio (0.0 - 1.0) - how much space first child takes
        ratio: f32,
    },
}

impl SplitNode {
    /// Create a new leaf node with given ID
    pub fn new_leaf(id: usize) -> Self {
        SplitNode::Leaf { id, ratio: 1.0 }
    }

    /// Create a new split node
    pub fn new_split(
        direction: SplitDirection,
        first: SplitNode,
        second: SplitNode,
        ratio: f32,
    ) -> Self {
        SplitNode::Split {
            direction,
            first: Box::new(first),
            second: Box::new(second),
            ratio: ratio.clamp(0.1, 0.9), // Prevent too small splits
        }
    }

    /// Get all leaf IDs in this subtree
    pub fn get_leaf_ids(&self) -> Vec<usize> {
        match self {
            SplitNode::Leaf { id, .. } => vec![*id],
            SplitNode::Split { first, second, .. } => {
                let mut ids = first.get_leaf_ids();
                ids.extend(second.get_leaf_ids());
                ids
            }
        }
    }

    /// Find a leaf node by ID and return mutable reference
    pub fn find_leaf_mut(&mut self, target_id: usize) -> Option<&mut SplitNode> {
        match self {
            SplitNode::Leaf { id, .. } if *id == target_id => Some(self),
            SplitNode::Leaf { .. } => None,
            SplitNode::Split { first, second, .. } => first
                .find_leaf_mut(target_id)
                .or_else(|| second.find_leaf_mut(target_id)),
        }
    }

    /// Check if this node contains a leaf with the given ID
    fn contains_leaf(&self, target_id: usize) -> bool {
        match self {
            SplitNode::Leaf { id, .. } => *id == target_id,
            SplitNode::Split { first, second, .. } => {
                first.contains_leaf(target_id) || second.contains_leaf(target_id)
            }
        }
    }

    /// Count total number of leaves
    pub fn leaf_count(&self) -> usize {
        match self {
            SplitNode::Leaf { .. } => 1,
            SplitNode::Split { first, second, .. } => first.leaf_count() + second.leaf_count(),
        }
    }

    /// Calculate the bounding box for a specific leaf
    pub fn calculate_bounds(
        &self,
        target_id: usize,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Option<(f32, f32, f32, f32)> {
        match self {
            SplitNode::Leaf { id, .. } if *id == target_id => Some((x, y, width, height)),
            SplitNode::Leaf { .. } => None,
            SplitNode::Split {
                direction,
                first,
                second,
                ratio,
                ..
            } => match direction {
                SplitDirection::Horizontal => {
                    let first_height = height * ratio;
                    let second_height = height * (1.0 - ratio);

                    first
                        .calculate_bounds(target_id, x, y, width, first_height)
                        .or_else(|| {
                            second.calculate_bounds(
                                target_id,
                                x,
                                y + first_height,
                                width,
                                second_height,
                            )
                        })
                }
                SplitDirection::Vertical => {
                    let first_width = width * ratio;
                    let second_width = width * (1.0 - ratio);

                    first
                        .calculate_bounds(target_id, x, y, first_width, height)
                        .or_else(|| {
                            second.calculate_bounds(
                                target_id,
                                x + first_width,
                                y,
                                second_width,
                                height,
                            )
                        })
                }
            },
        }
    }
}

/// Container for managing split terminals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitContainer {
    /// Root of the split tree
    root: SplitNode,
    /// Currently focused pane ID
    focused_id: usize,
    /// Next available pane ID
    next_id: usize,
}

impl SplitContainer {
    /// Create a new split container with a single pane
    pub fn new() -> Self {
        Self {
            root: SplitNode::new_leaf(0),
            focused_id: 0,
            next_id: 1,
        }
    }

    /// Get the focused pane ID
    pub fn focused_id(&self) -> usize {
        self.focused_id
    }

    /// Set the focused pane ID
    pub fn set_focused_id(&mut self, id: usize) -> bool {
        if self.root.get_leaf_ids().contains(&id) {
            self.focused_id = id;
            true
        } else {
            false
        }
    }

    /// Get all pane IDs
    pub fn get_all_ids(&self) -> Vec<usize> {
        self.root.get_leaf_ids()
    }

    /// Get the number of panes
    pub fn pane_count(&self) -> usize {
        self.root.leaf_count()
    }

    /// Split the focused pane in the given direction
    pub fn split_focused(&mut self, direction: SplitDirection) -> usize {
        let new_id = self.next_id;
        self.next_id += 1;

        self.split_pane(self.focused_id, direction, new_id);
        self.focused_id = new_id;
        new_id
    }

    /// Split a specific pane
    fn split_pane(&mut self, target_id: usize, direction: SplitDirection, new_id: usize) {
        Self::split_node_recursive(&mut self.root, target_id, direction, new_id);
    }

    /// Recursively split a node in the tree
    fn split_node_recursive(
        node: &mut SplitNode,
        target_id: usize,
        direction: SplitDirection,
        new_id: usize,
    ) -> bool {
        match node {
            SplitNode::Leaf { id, .. } if *id == target_id => {
                // Replace this leaf with a split
                let old_node = node.clone();
                *node = SplitNode::new_split(direction, old_node, SplitNode::new_leaf(new_id), 0.5);
                true
            }
            SplitNode::Leaf { .. } => false,
            SplitNode::Split { first, second, .. } => {
                Self::split_node_recursive(first, target_id, direction, new_id)
                    || Self::split_node_recursive(second, target_id, direction, new_id)
            }
        }
    }

    /// Close a pane by ID
    pub fn close_pane(&mut self, target_id: usize) -> bool {
        // Cannot close the last pane
        if self.pane_count() == 1 {
            return false;
        }

        // If closing focused pane, move focus first
        if target_id == self.focused_id {
            let ids = self.get_all_ids();
            if let Some(new_focus) = ids.iter().find(|&&id| id != target_id) {
                self.focused_id = *new_focus;
            }
        }

        self.remove_pane_from_tree(target_id)
    }

    /// Remove a pane from the tree
    fn remove_pane_from_tree(&mut self, target_id: usize) -> bool {
        // Handle root split specially
        if let SplitNode::Split { first, second, .. } = &self.root {
            match (first.as_ref(), second.as_ref()) {
                (SplitNode::Leaf { id, .. }, _) if *id == target_id => {
                    // Replace root with second child
                    self.root = (**second).clone();
                    return true;
                }
                (_, SplitNode::Leaf { id, .. }) if *id == target_id => {
                    // Replace root with first child
                    self.root = (**first).clone();
                    return true;
                }
                _ => {}
            }
        }

        // Find grandparent and replace parent with sibling
        let mut root_clone = self.root.clone();
        if Self::remove_from_subtree_inner(&mut root_clone, target_id) {
            self.root = root_clone;
            return true;
        }
        false
    }

    /// Recursively remove a pane from subtree
    fn remove_from_subtree_inner(node: &mut SplitNode, target_id: usize) -> bool {
        if let SplitNode::Split { first, second, .. } = node {
            // Check if either child is a split containing the target
            match (first.as_mut(), second.as_mut()) {
                (
                    SplitNode::Split {
                        first: f1,
                        second: s1,
                        ..
                    },
                    _,
                ) => {
                    match (f1.as_ref(), s1.as_ref()) {
                        (SplitNode::Leaf { id, .. }, _) if *id == target_id => {
                            // Replace first child with its second child
                            *first = s1.clone();
                            return true;
                        }
                        (_, SplitNode::Leaf { id, .. }) if *id == target_id => {
                            // Replace first child with its first child
                            *first = f1.clone();
                            return true;
                        }
                        _ => {}
                    }
                }
                (
                    _,
                    SplitNode::Split {
                        first: f2,
                        second: s2,
                        ..
                    },
                ) => {
                    match (f2.as_ref(), s2.as_ref()) {
                        (SplitNode::Leaf { id, .. }, _) if *id == target_id => {
                            // Replace second child with its second child
                            *second = s2.clone();
                            return true;
                        }
                        (_, SplitNode::Leaf { id, .. }) if *id == target_id => {
                            // Replace second child with its first child
                            *second = f2.clone();
                            return true;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }

            // Recursively search
            if Self::remove_from_subtree_inner(first, target_id) {
                return true;
            }
            if Self::remove_from_subtree_inner(second, target_id) {
                return true;
            }
        }

        false
    }

    /// Navigate focus in the given direction
    pub fn navigate_focus(&mut self, direction: NavigationDirection) -> bool {
        let current_bounds = self.get_pane_bounds(self.focused_id);
        if current_bounds.is_none() {
            return false;
        }

        let (cx, cy, cw, ch) = current_bounds.unwrap();
        let center_x = cx + cw / 2.0;
        let center_y = cy + ch / 2.0;

        // Find the nearest pane in the given direction
        let mut best_candidate: Option<(usize, f32)> = None;

        for id in self.get_all_ids() {
            if id == self.focused_id {
                continue;
            }

            if let Some((x, y, w, h)) = self.get_pane_bounds(id) {
                let target_x = x + w / 2.0;
                let target_y = y + h / 2.0;

                let is_valid = match direction {
                    NavigationDirection::Up => target_y < center_y,
                    NavigationDirection::Down => target_y > center_y,
                    NavigationDirection::Left => target_x < center_x,
                    NavigationDirection::Right => target_x > center_x,
                };

                if is_valid {
                    let distance =
                        ((target_x - center_x).powi(2) + (target_y - center_y).powi(2)).sqrt();

                    if let Some((_, best_dist)) = best_candidate {
                        if distance < best_dist {
                            best_candidate = Some((id, distance));
                        }
                    } else {
                        best_candidate = Some((id, distance));
                    }
                }
            }
        }

        if let Some((new_id, _)) = best_candidate {
            self.focused_id = new_id;
            true
        } else {
            false
        }
    }

    /// Get the bounding box for a pane (x, y, width, height) in normalized coordinates (0.0-1.0)
    pub fn get_pane_bounds(&self, id: usize) -> Option<(f32, f32, f32, f32)> {
        self.root.calculate_bounds(id, 0.0, 0.0, 1.0, 1.0)
    }

    /// Resize a split by adjusting the ratio
    pub fn resize_split(&mut self, pane_id: usize, delta: f32) -> bool {
        Self::adjust_split_ratio_recursive(&mut self.root, pane_id, delta)
    }

    /// Recursively adjust split ratio for a pane
    fn adjust_split_ratio_recursive(node: &mut SplitNode, pane_id: usize, delta: f32) -> bool {
        if let SplitNode::Split {
            first,
            second,
            ratio,
            ..
        } = node
        {
            // Check if the pane is directly in this split
            let first_has_pane = first.contains_leaf(pane_id);
            let second_has_pane = second.contains_leaf(pane_id);

            if first_has_pane && !second_has_pane {
                // Increase first's ratio
                *ratio = (*ratio + delta).clamp(0.1, 0.9);
                return true;
            } else if !first_has_pane && second_has_pane {
                // Decrease first's ratio (increase second's)
                *ratio = (*ratio - delta).clamp(0.1, 0.9);
                return true;
            }

            // Recursively search
            if Self::adjust_split_ratio_recursive(first, pane_id, delta) {
                return true;
            }
            if Self::adjust_split_ratio_recursive(second, pane_id, delta) {
                return true;
            }
        }

        false
    }

    /// Get the root node (for rendering)
    pub fn root(&self) -> &SplitNode {
        &self.root
    }
}

impl Default for SplitContainer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_container() {
        let container = SplitContainer::new();
        assert_eq!(container.pane_count(), 1);
        assert_eq!(container.focused_id(), 0);
        assert_eq!(container.get_all_ids(), vec![0]);
    }

    #[test]
    fn test_split_horizontal() {
        let mut container = SplitContainer::new();
        let new_id = container.split_focused(SplitDirection::Horizontal);

        assert_eq!(container.pane_count(), 2);
        assert_eq!(new_id, 1);
        assert_eq!(container.focused_id(), 1);

        let ids = container.get_all_ids();
        assert!(ids.contains(&0));
        assert!(ids.contains(&1));
    }

    #[test]
    fn test_split_vertical() {
        let mut container = SplitContainer::new();
        let new_id = container.split_focused(SplitDirection::Vertical);

        assert_eq!(container.pane_count(), 2);
        assert_eq!(new_id, 1);
        assert_eq!(container.focused_id(), 1);
    }

    #[test]
    fn test_multiple_splits() {
        let mut container = SplitContainer::new();

        // Split horizontally: [0] -> [0][1]
        container.split_focused(SplitDirection::Horizontal);
        assert_eq!(container.pane_count(), 2);

        // Split vertically on pane 1: [0][1] -> [0][1|2]
        let id2 = container.split_focused(SplitDirection::Vertical);
        assert_eq!(container.pane_count(), 3);
        assert_eq!(id2, 2);

        let ids = container.get_all_ids();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&0));
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
    }

    #[test]
    fn test_close_pane() {
        let mut container = SplitContainer::new();

        // Cannot close the last pane
        assert!(!container.close_pane(0));
        assert_eq!(container.pane_count(), 1);

        // Create splits
        container.split_focused(SplitDirection::Horizontal);
        container.split_focused(SplitDirection::Vertical);
        assert_eq!(container.pane_count(), 3);

        // Close a pane
        assert!(container.close_pane(2));
        assert_eq!(container.pane_count(), 2);

        // Verify remaining panes
        let ids = container.get_all_ids();
        assert!(ids.contains(&0));
        assert!(ids.contains(&1));
        assert!(!ids.contains(&2));
    }

    #[test]
    fn test_close_focused_pane_moves_focus() {
        let mut container = SplitContainer::new();
        container.split_focused(SplitDirection::Horizontal);

        let focused = container.focused_id();
        assert_eq!(focused, 1);

        // Close focused pane
        container.close_pane(focused);

        // Focus should have moved
        assert_ne!(container.focused_id(), 1);
        assert_eq!(container.focused_id(), 0);
    }

    #[test]
    fn test_set_focused_id() {
        let mut container = SplitContainer::new();
        container.split_focused(SplitDirection::Horizontal);

        // Valid ID
        assert!(container.set_focused_id(0));
        assert_eq!(container.focused_id(), 0);

        // Invalid ID
        assert!(!container.set_focused_id(999));
        assert_eq!(container.focused_id(), 0); // Unchanged
    }

    #[test]
    fn test_pane_bounds_single() {
        let container = SplitContainer::new();
        let bounds = container.get_pane_bounds(0);

        assert!(bounds.is_some());
        let (x, y, w, h) = bounds.unwrap();
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
        assert_eq!(w, 1.0);
        assert_eq!(h, 1.0);
    }

    #[test]
    fn test_pane_bounds_horizontal_split() {
        let mut container = SplitContainer::new();
        container.split_focused(SplitDirection::Horizontal);

        let bounds0 = container.get_pane_bounds(0).unwrap();
        let bounds1 = container.get_pane_bounds(1).unwrap();

        // Pane 0 should be on top
        assert_eq!(bounds0.0, 0.0); // x
        assert_eq!(bounds0.1, 0.0); // y
        assert_eq!(bounds0.2, 1.0); // width
        assert!(bounds0.3 > 0.0 && bounds0.3 < 1.0); // height

        // Pane 1 should be on bottom
        assert_eq!(bounds1.0, 0.0); // x
        assert!(bounds1.1 > 0.0); // y
        assert_eq!(bounds1.2, 1.0); // width
        assert!(bounds1.3 > 0.0 && bounds1.3 < 1.0); // height

        // Heights should sum to 1.0
        assert!((bounds0.3 + bounds1.3 - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_pane_bounds_vertical_split() {
        let mut container = SplitContainer::new();
        container.split_focused(SplitDirection::Vertical);

        let bounds0 = container.get_pane_bounds(0).unwrap();
        let bounds1 = container.get_pane_bounds(1).unwrap();

        // Pane 0 should be on left
        assert_eq!(bounds0.0, 0.0); // x
        assert_eq!(bounds0.1, 0.0); // y
        assert!(bounds0.2 > 0.0 && bounds0.2 < 1.0); // width
        assert_eq!(bounds0.3, 1.0); // height

        // Pane 1 should be on right
        assert!(bounds1.0 > 0.0); // x
        assert_eq!(bounds1.1, 0.0); // y
        assert!(bounds1.2 > 0.0 && bounds1.2 < 1.0); // width
        assert_eq!(bounds1.3, 1.0); // height

        // Widths should sum to 1.0
        assert!((bounds0.2 + bounds1.2 - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_navigate_focus_horizontal() {
        let mut container = SplitContainer::new();
        container.split_focused(SplitDirection::Horizontal);

        // Start at pane 1 (bottom)
        assert_eq!(container.focused_id(), 1);

        // Navigate up
        assert!(container.navigate_focus(NavigationDirection::Up));
        assert_eq!(container.focused_id(), 0);

        // Navigate down
        assert!(container.navigate_focus(NavigationDirection::Down));
        assert_eq!(container.focused_id(), 1);

        // Cannot navigate left/right
        assert!(!container.navigate_focus(NavigationDirection::Left));
        assert!(!container.navigate_focus(NavigationDirection::Right));
    }

    #[test]
    fn test_navigate_focus_vertical() {
        let mut container = SplitContainer::new();
        container.split_focused(SplitDirection::Vertical);

        // Start at pane 1 (right)
        assert_eq!(container.focused_id(), 1);

        // Navigate left
        assert!(container.navigate_focus(NavigationDirection::Left));
        assert_eq!(container.focused_id(), 0);

        // Navigate right
        assert!(container.navigate_focus(NavigationDirection::Right));
        assert_eq!(container.focused_id(), 1);

        // Cannot navigate up/down
        assert!(!container.navigate_focus(NavigationDirection::Up));
        assert!(!container.navigate_focus(NavigationDirection::Down));
    }

    #[test]
    fn test_navigate_focus_complex() {
        let mut container = SplitContainer::new();

        // Create a complex layout:
        //   [0][1]
        //   [0][2]
        container.split_focused(SplitDirection::Vertical); // [0][1]
        container.split_focused(SplitDirection::Horizontal); // [0][1|2]

        // Currently at pane 2
        assert_eq!(container.focused_id(), 2);

        // Navigate up to pane 1
        assert!(container.navigate_focus(NavigationDirection::Up));
        assert_eq!(container.focused_id(), 1);

        // Navigate left to pane 0
        assert!(container.navigate_focus(NavigationDirection::Left));
        assert_eq!(container.focused_id(), 0);
    }

    #[test]
    fn test_resize_split() {
        let mut container = SplitContainer::new();
        container.split_focused(SplitDirection::Vertical);

        // Get initial bounds
        let initial_bounds = container.get_pane_bounds(0).unwrap();
        let initial_width = initial_bounds.2;

        // Resize pane 0 to be larger
        assert!(container.resize_split(0, 0.1));

        let new_bounds = container.get_pane_bounds(0).unwrap();
        let new_width = new_bounds.2;

        // Width should have increased
        assert!(new_width > initial_width);
    }

    #[test]
    fn test_resize_split_clamps() {
        let mut container = SplitContainer::new();
        container.split_focused(SplitDirection::Vertical);

        // Try to resize to extreme values
        for _ in 0..20 {
            container.resize_split(0, 0.1);
        }

        let bounds = container.get_pane_bounds(0).unwrap();
        // Should be clamped to maximum (0.9)
        assert!(bounds.2 <= 0.9);
        assert!(bounds.2 >= 0.85); // Should be close to 0.9
    }

    #[test]
    fn test_serialize_deserialize() {
        let mut container = SplitContainer::new();
        container.split_focused(SplitDirection::Horizontal);
        container.split_focused(SplitDirection::Vertical);

        // Serialize
        let json = serde_json::to_string(&container).unwrap();

        // Deserialize
        let deserialized: SplitContainer = serde_json::from_str(&json).unwrap();

        // Verify structure
        assert_eq!(deserialized.pane_count(), container.pane_count());
        assert_eq!(deserialized.focused_id(), container.focused_id());
        assert_eq!(deserialized.get_all_ids(), container.get_all_ids());
    }

    #[test]
    fn test_split_direction_display() {
        assert_eq!(format!("{}", SplitDirection::Horizontal), "Horizontal");
        assert_eq!(format!("{}", SplitDirection::Vertical), "Vertical");
    }

    #[test]
    fn test_leaf_count() {
        let mut container = SplitContainer::new();
        assert_eq!(container.root().leaf_count(), 1);

        container.split_focused(SplitDirection::Horizontal);
        assert_eq!(container.root().leaf_count(), 2);

        container.split_focused(SplitDirection::Vertical);
        assert_eq!(container.root().leaf_count(), 3);
    }

    #[test]
    fn test_complex_nested_splits() {
        let mut container = SplitContainer::new();

        // Create a complex 4-pane layout
        container.split_focused(SplitDirection::Horizontal); // [0][1]
        container.set_focused_id(0);
        container.split_focused(SplitDirection::Vertical); // [0|2][1]
        container.set_focused_id(1);
        container.split_focused(SplitDirection::Vertical); // [0|2][1|3]

        assert_eq!(container.pane_count(), 4);

        // Verify all panes have valid bounds
        for id in container.get_all_ids() {
            let bounds = container.get_pane_bounds(id);
            assert!(bounds.is_some(), "Pane {} should have bounds", id);
        }
    }
}

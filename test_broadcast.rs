// Standalone test for broadcast module
// Run with: rustc --test test_broadcast.rs && ./test_broadcast

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

// Minimal types needed for testing
type Uuid = u128;

fn new_uuid() -> Uuid {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BroadcastMode {
    Full,
    Selective,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BroadcastTrigger {
    ctrl: bool,
    alt: bool,
    shift: bool,
    command: bool,
}

impl BroadcastTrigger {
    fn default_selective() -> Self {
        Self {
            ctrl: true,
            alt: false,
            shift: true,
            command: false,
        }
    }

    fn matches(&self, ctrl: bool, alt: bool, shift: bool, command: bool) -> bool {
        self.ctrl == ctrl && self.alt == alt && self.shift == shift && self.command == command
    }
}

#[derive(Debug, Clone)]
struct BroadcastGroup {
    name: String,
    members: HashSet<Uuid>,
    active: bool,
    mode: BroadcastMode,
    trigger: BroadcastTrigger,
}

impl BroadcastGroup {
    fn new(name: String) -> Self {
        Self {
            name,
            members: HashSet::new(),
            active: false,
            mode: BroadcastMode::Full,
            trigger: BroadcastTrigger::default_selective(),
        }
    }

    fn add_terminal(&mut self, terminal_id: Uuid) -> bool {
        self.members.insert(terminal_id)
    }

    fn contains(&self, terminal_id: &Uuid) -> bool {
        self.members.contains(terminal_id)
    }

    fn activate(&mut self) {
        self.active = true;
    }

    fn deactivate(&mut self) {
        self.active = false;
    }

    fn should_broadcast(&self, ctrl: bool, alt: bool, shift: bool, command: bool) -> bool {
        if !self.active {
            return false;
        }

        match self.mode {
            BroadcastMode::Full => true,
            BroadcastMode::Selective => self.trigger.matches(ctrl, alt, shift, command),
            BroadcastMode::Disabled => false,
        }
    }
}

#[derive(Debug, Clone)]
struct BroadcastManager {
    groups: HashMap<String, BroadcastGroup>,
    active_group: Option<String>,
    known_terminals: HashSet<Uuid>,
}

impl BroadcastManager {
    fn new() -> Self {
        Self {
            groups: HashMap::new(),
            active_group: None,
            known_terminals: HashSet::new(),
        }
    }

    fn register_terminal(&mut self, terminal_id: Uuid) {
        self.known_terminals.insert(terminal_id);
    }

    fn create_group(&mut self, name: String) -> Result<(), String> {
        if self.groups.contains_key(&name) {
            return Err(format!("Group already exists: {}", name));
        }
        self.groups.insert(name.clone(), BroadcastGroup::new(name));
        Ok(())
    }

    fn add_to_group(&mut self, group_name: &str, terminal_id: Uuid) -> Result<(), String> {
        let group = self
            .groups
            .get_mut(group_name)
            .ok_or_else(|| format!("Group not found: {}", group_name))?;
        group.add_terminal(terminal_id);
        Ok(())
    }

    fn activate_group(&mut self, name: &str) -> Result<(), String> {
        // Deactivate current group if any
        if let Some(active_name) = self.active_group.take() {
            if let Some(group) = self.groups.get_mut(&active_name) {
                group.deactivate();
            }
        }

        // Activate new group
        let group = self
            .groups
            .get_mut(name)
            .ok_or_else(|| format!("Group not found: {}", name))?;
        group.activate();
        self.active_group = Some(name.to_string());
        Ok(())
    }

    fn get_broadcast_targets(
        &self,
        source_terminal: &Uuid,
        ctrl: bool,
        alt: bool,
        shift: bool,
        command: bool,
    ) -> Option<Vec<Uuid>> {
        let active_name = self.active_group.as_ref()?;
        let group = self.groups.get(active_name)?;

        if !group.contains(source_terminal) {
            return None;
        }

        if !group.should_broadcast(ctrl, alt, shift, command) {
            return None;
        }

        Some(
            group
                .members
                .iter()
                .filter(|id| *id != source_terminal)
                .copied()
                .collect(),
        )
    }
}

// Tests
#[test]
fn test_basic_broadcast() {
    let mut manager = BroadcastManager::new();

    let term1 = new_uuid();
    let term2 = new_uuid();
    let term3 = new_uuid();

    manager.register_terminal(term1);
    manager.register_terminal(term2);
    manager.register_terminal(term3);

    manager.create_group("test".to_string()).unwrap();
    manager.add_to_group("test", term1).unwrap();
    manager.add_to_group("test", term2).unwrap();
    manager.add_to_group("test", term3).unwrap();

    manager.activate_group("test").unwrap();

    let targets = manager
        .get_broadcast_targets(&term1, false, false, false, false)
        .unwrap();

    assert_eq!(targets.len(), 2);
    assert!(!targets.contains(&term1));
    assert!(targets.contains(&term2));
    assert!(targets.contains(&term3));

    println!("✓ Basic broadcast test passed");
}

#[test]
fn test_selective_mode() {
    let mut manager = BroadcastManager::new();

    let term1 = new_uuid();
    let term2 = new_uuid();

    manager.register_terminal(term1);
    manager.register_terminal(term2);

    manager.create_group("test".to_string()).unwrap();
    manager.add_to_group("test", term1).unwrap();
    manager.add_to_group("test", term2).unwrap();

    manager.activate_group("test").unwrap();

    // Change to selective mode
    if let Some(group) = manager.groups.get_mut("test") {
        group.mode = BroadcastMode::Selective;
    }

    // Without trigger keys - should not broadcast
    let targets = manager.get_broadcast_targets(&term1, false, false, false, false);
    assert!(targets.is_none());

    // With trigger keys (Ctrl+Shift) - should broadcast
    let targets = manager
        .get_broadcast_targets(&term1, true, false, true, false)
        .unwrap();
    assert_eq!(targets.len(), 1);
    assert!(targets.contains(&term2));

    println!("✓ Selective mode test passed");
}

#[test]
fn test_disabled_mode() {
    let mut manager = BroadcastManager::new();

    let term1 = new_uuid();
    let term2 = new_uuid();

    manager.register_terminal(term1);
    manager.register_terminal(term2);

    manager.create_group("test".to_string()).unwrap();
    manager.add_to_group("test", term1).unwrap();
    manager.add_to_group("test", term2).unwrap();

    manager.activate_group("test").unwrap();

    // Change to disabled mode
    if let Some(group) = manager.groups.get_mut("test") {
        group.mode = BroadcastMode::Disabled;
    }

    // Should never broadcast in disabled mode
    let targets = manager.get_broadcast_targets(&term1, false, false, false, false);
    assert!(targets.is_none());

    let targets = manager.get_broadcast_targets(&term1, true, true, true, true);
    assert!(targets.is_none());

    println!("✓ Disabled mode test passed");
}

fn main() {
    println!("Running broadcast module tests...\n");

    test_basic_broadcast();
    test_selective_mode();
    test_disabled_mode();

    println!("\n✓ All tests passed!");
}

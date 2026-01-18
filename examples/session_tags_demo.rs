//! Session Tags Demo
//!
//! Demonstrates the session tagging system in AgTerm.
//!
//! Run with: `cargo run --example session_tags_demo`

use agterm::session_tags::{SessionTagManager, TagUpdate};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AgTerm Session Tags Demo ===\n");

    // Create a new session tag manager with default tags
    let mut manager = SessionTagManager::with_defaults();

    println!("1. Default Tags:");
    for tag in manager.tag_manager().list_tags() {
        println!(
            "   - {} (RGB: {:?}) {}",
            tag.name,
            tag.color,
            tag.icon.as_ref().unwrap_or(&String::from(""))
        );
    }

    // Create a custom tag
    println!("\n2. Creating custom tag 'important':");
    manager
        .tag_manager_mut()
        .create_tag(
            "important".to_string(),
            (255, 215, 0), // Gold
            Some("⭐".to_string()),
        )?;
    println!("   ✓ Tag 'important' created");

    // Tag some sessions
    println!("\n3. Tagging sessions:");
    manager.tag_session("session-001".to_string(), "work".to_string())?;
    manager.tag_session("session-001".to_string(), "important".to_string())?;
    println!("   ✓ Tagged session-001 with 'work' and 'important'");

    manager.tag_session("session-002".to_string(), "personal".to_string())?;
    manager.tag_session("session-002".to_string(), "dev".to_string())?;
    println!("   ✓ Tagged session-002 with 'personal' and 'dev'");

    manager.tag_session("session-003".to_string(), "urgent".to_string())?;
    println!("   ✓ Tagged session-003 with 'urgent'");

    // Add notes to sessions
    println!("\n4. Adding notes:");
    manager.set_session_note(
        "session-001".to_string(),
        Some("Main project work session".to_string()),
    )?;
    println!("   ✓ Added note to session-001");

    manager.set_session_note(
        "session-003".to_string(),
        Some("Production hotfix - needs immediate attention".to_string()),
    )?;
    println!("   ✓ Added note to session-003");

    // Pin a session
    println!("\n5. Pinning sessions:");
    manager.pin_session("session-001".to_string())?;
    println!("   ✓ Pinned session-001");

    // Query sessions by tag
    println!("\n6. Querying sessions by tag:");
    let work_sessions = manager.get_sessions_by_tag("work");
    println!("   Sessions tagged with 'work': {}", work_sessions.len());
    for session in work_sessions {
        println!("      - {}", session.session_id);
    }

    let urgent_sessions = manager.get_sessions_by_tag("urgent");
    println!("   Sessions tagged with 'urgent': {}", urgent_sessions.len());
    for session in urgent_sessions {
        println!("      - {}", session.session_id);
        if let Some(notes) = &session.notes {
            println!("        Notes: {}", notes);
        }
    }

    // Get pinned sessions
    println!("\n7. Pinned sessions:");
    let pinned = manager.get_pinned_sessions();
    println!("   Total pinned: {}", pinned.len());
    for session in pinned {
        let tags = manager.get_session_tags(&session.session_id);
        let tag_names: Vec<&str> = tags.iter().map(|t| t.name.as_str()).collect();
        println!(
            "      - {} [{}]",
            session.session_id,
            tag_names.join(", ")
        );
    }

    // Search sessions
    println!("\n8. Searching sessions:");
    let search_results = manager.search_sessions("prod");
    println!("   Search results for 'prod': {}", search_results.len());
    for session in search_results {
        println!("      - {}", session.session_id);
        if let Some(notes) = &session.notes {
            println!("        Notes: {}", notes);
        }
    }

    // Update a tag
    println!("\n9. Updating tag:");
    let update = TagUpdate::new()
        .with_color((255, 100, 100))
        .with_description(Some("High priority tasks".to_string()));
    manager.tag_manager_mut().update_tag("important", update)?;
    println!("   ✓ Updated 'important' tag");

    // Get details for a specific session
    println!("\n10. Session details:");
    if let Some(session) = manager.get_session("session-001") {
        println!("   Session ID: {}", session.session_id);
        println!("   Pinned: {}", session.pinned);
        println!("   Tags: {:?}", session.tags);
        if let Some(notes) = &session.notes {
            println!("   Notes: {}", notes);
        }
        println!("   Created: {}", session.created_at);
        println!("   Last accessed: {}", session.last_accessed);
    }

    // Save to file
    println!("\n11. Saving to file:");
    let temp_path = PathBuf::from("/tmp/agterm_session_tags_demo.json");
    manager.save_to_file(&temp_path)?;
    println!("   ✓ Saved to {:?}", temp_path);

    // Load from file
    println!("\n12. Loading from file:");
    let loaded_manager = SessionTagManager::load_from_file(&temp_path)?;
    println!("   ✓ Loaded successfully");
    println!(
        "   Tags: {}",
        loaded_manager.tag_manager().list_tags().len()
    );
    println!(
        "   Sessions: {}",
        loaded_manager
            .search_sessions("")
            .len()
    );

    // Clean up
    std::fs::remove_file(&temp_path)?;
    println!("\n✓ Demo completed successfully!");

    Ok(())
}

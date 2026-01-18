//! Integration test for session restoration

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_session_serialization() {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TabState {
        cwd: String,
        title: Option<String>,
        id: usize,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct SessionState {
        tabs: Vec<TabState>,
        active_tab: usize,
        window_size: Option<(u16, u16)>,
        font_size: f32,
    }

    // Create a sample session
    let session = SessionState {
        tabs: vec![
            TabState {
                cwd: "/home/user/project1".to_string(),
                title: Some("Project 1".to_string()),
                id: 0,
            },
            TabState {
                cwd: "/home/user/project2".to_string(),
                title: None,
                id: 1,
            },
        ],
        active_tab: 1,
        window_size: Some((120, 40)),
        font_size: 14.0,
    };

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&session).unwrap();
    println!("Serialized session:\n{}", json);

    // Deserialize back
    let restored: SessionState = serde_json::from_str(&json).unwrap();

    // Verify fields
    assert_eq!(restored.tabs.len(), 2);
    assert_eq!(restored.active_tab, 1);
    assert_eq!(restored.font_size, 14.0);
    assert_eq!(restored.tabs[0].cwd, "/home/user/project1");
    assert_eq!(restored.tabs[0].title, Some("Project 1".to_string()));
    assert_eq!(restored.tabs[1].title, None);
}

#[test]
fn test_session_file_io() {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TabState {
        cwd: String,
        title: Option<String>,
        id: usize,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct SessionState {
        tabs: Vec<TabState>,
        active_tab: usize,
        window_size: Option<(u16, u16)>,
        font_size: f32,
    }

    impl SessionState {
        fn save_to_file(&self, path: &std::path::Path) -> Result<(), std::io::Error> {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let json = serde_json::to_string_pretty(self)?;
            std::fs::write(path, json)?;
            Ok(())
        }

        fn load_from_file(path: &std::path::Path) -> Result<Self, std::io::Error> {
            let json = std::fs::read_to_string(path)?;
            let state = serde_json::from_str(&json)?;
            Ok(state)
        }
    }

    let temp_dir = TempDir::new().unwrap();
    let session_path = temp_dir.path().join("session.json");

    // Create and save a session
    let session = SessionState {
        tabs: vec![
            TabState {
                cwd: "/tmp/test1".to_string(),
                title: Some("Test Tab 1".to_string()),
                id: 0,
            },
            TabState {
                cwd: "/tmp/test2".to_string(),
                title: Some("Test Tab 2".to_string()),
                id: 1,
            },
            TabState {
                cwd: "/tmp/test3".to_string(),
                title: None,
                id: 2,
            },
        ],
        active_tab: 1,
        window_size: Some((120, 40)),
        font_size: 16.0,
    };

    // Save to file
    session.save_to_file(&session_path).unwrap();
    assert!(session_path.exists());

    // Load from file
    let loaded = SessionState::load_from_file(&session_path).unwrap();

    // Verify
    assert_eq!(loaded.tabs.len(), 3);
    assert_eq!(loaded.active_tab, 1);
    assert_eq!(loaded.font_size, 16.0);
    assert_eq!(loaded.tabs[0].cwd, "/tmp/test1");
    assert_eq!(loaded.tabs[1].title, Some("Test Tab 2".to_string()));
    assert_eq!(loaded.tabs[2].title, None);
}

#[test]
fn test_session_empty_tabs() {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TabState {
        cwd: String,
        title: Option<String>,
        id: usize,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct SessionState {
        tabs: Vec<TabState>,
        active_tab: usize,
        window_size: Option<(u16, u16)>,
        font_size: f32,
    }

    // Session with no tabs
    let session = SessionState {
        tabs: vec![],
        active_tab: 0,
        window_size: None,
        font_size: 14.0,
    };

    let json = serde_json::to_string_pretty(&session).unwrap();
    let restored: SessionState = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.tabs.len(), 0);
    assert_eq!(restored.active_tab, 0);
}

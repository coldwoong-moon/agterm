//! Code snippet system for AgTerm
//!
//! Provides a powerful snippet system with:
//! - Snippet definitions with triggers (abbreviations)
//! - Template expansion with placeholder support
//! - Category-based organization
//! - CRUD operations for snippet management
//! - Autocomplete integration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Represents a code snippet with template and metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Snippet {
    /// Unique identifier for the snippet
    pub id: String,
    /// Display name of the snippet
    pub name: String,
    /// Description of what the snippet does
    pub description: String,
    /// Trigger abbreviation (e.g., "fn" for function)
    pub trigger: String,
    /// Template content with placeholders
    pub template: String,
    /// Category for grouping (e.g., "rust", "bash", "git")
    pub category: String,
    /// Additional tags for search/filtering
    pub tags: Vec<String>,
}

impl Snippet {
    /// Create a new snippet
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        trigger: impl Into<String>,
        template: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        let trigger = trigger.into();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            description: description.into(),
            trigger: trigger.clone(),
            template: template.into(),
            category: category.into(),
            tags: Vec::new(),
        }
    }

    /// Add a tag to the snippet
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add multiple tags to the snippet
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags.extend(tags);
        self
    }
}

/// Represents a placeholder in a snippet template
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Placeholder {
    /// Sequential placeholder: $1, $2, etc.
    Sequential(usize),
    /// Named placeholder with optional default: ${name:default}
    Named {
        name: String,
        default: Option<String>,
    },
    /// Final cursor position: $0
    Final,
}

/// Result of parsing a snippet template
#[derive(Debug, Clone)]
pub struct ParsedTemplate {
    /// Template parts interspersed with placeholders
    pub parts: Vec<TemplatePart>,
    /// Ordered list of placeholders (excluding $0)
    pub placeholders: Vec<Placeholder>,
    /// Index of the final cursor position in parts
    pub final_position: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplatePart {
    Text(String),
    Placeholder(Placeholder),
}

/// Manages snippets and provides CRUD operations
pub struct SnippetManager {
    snippets: HashMap<String, Snippet>,
    trigger_index: HashMap<String, Vec<String>>, // trigger -> snippet IDs
    category_index: HashMap<String, Vec<String>>, // category -> snippet IDs
}

impl SnippetManager {
    /// Create a new snippet manager
    pub fn new() -> Self {
        Self {
            snippets: HashMap::new(),
            trigger_index: HashMap::new(),
            category_index: HashMap::new(),
        }
    }

    /// Add a snippet to the manager
    pub fn add_snippet(&mut self, snippet: Snippet) -> Result<(), SnippetError> {
        // Check for duplicate trigger
        if let Some(existing_ids) = self.trigger_index.get(&snippet.trigger) {
            if !existing_ids.is_empty() {
                return Err(SnippetError::DuplicateTrigger(snippet.trigger.clone()));
            }
        }

        let id = snippet.id.clone();
        let trigger = snippet.trigger.clone();
        let category = snippet.category.clone();

        // Add to main storage
        self.snippets.insert(id.clone(), snippet);

        // Update trigger index
        self.trigger_index
            .entry(trigger)
            .or_default()
            .push(id.clone());

        // Update category index
        self.category_index
            .entry(category)
            .or_default()
            .push(id);

        Ok(())
    }

    /// Remove a snippet by ID
    pub fn remove_snippet(&mut self, id: &str) -> Result<Snippet, SnippetError> {
        let snippet = self
            .snippets
            .remove(id)
            .ok_or_else(|| SnippetError::NotFound(id.to_string()))?;

        // Remove from trigger index
        if let Some(ids) = self.trigger_index.get_mut(&snippet.trigger) {
            ids.retain(|i| i != id);
        }

        // Remove from category index
        if let Some(ids) = self.category_index.get_mut(&snippet.category) {
            ids.retain(|i| i != id);
        }

        Ok(snippet)
    }

    /// Update an existing snippet
    pub fn update_snippet(&mut self, id: &str, snippet: Snippet) -> Result<(), SnippetError> {
        if !self.snippets.contains_key(id) {
            return Err(SnippetError::NotFound(id.to_string()));
        }

        // Remove old snippet
        self.remove_snippet(id)?;

        // Add updated snippet (but keep the original ID)
        let mut updated = snippet;
        updated.id = id.to_string();
        self.add_snippet(updated)?;

        Ok(())
    }

    /// Get a snippet by ID
    pub fn get_snippet(&self, id: &str) -> Option<&Snippet> {
        self.snippets.get(id)
    }

    /// Find snippets by trigger prefix
    pub fn find_by_trigger(&self, prefix: &str) -> Vec<&Snippet> {
        let mut results = Vec::new();
        for (trigger, ids) in &self.trigger_index {
            if trigger.starts_with(prefix) {
                for id in ids {
                    if let Some(snippet) = self.snippets.get(id) {
                        results.push(snippet);
                    }
                }
            }
        }
        results.sort_by(|a, b| a.trigger.cmp(&b.trigger));
        results
    }

    /// Find snippets by exact trigger match
    pub fn find_exact_trigger(&self, trigger: &str) -> Option<&Snippet> {
        self.trigger_index
            .get(trigger)
            .and_then(|ids| ids.first())
            .and_then(|id| self.snippets.get(id))
    }

    /// Get all snippets in a category
    pub fn get_by_category(&self, category: &str) -> Vec<&Snippet> {
        self.category_index
            .get(category)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.snippets.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all categories
    pub fn get_categories(&self) -> Vec<String> {
        let mut categories: Vec<_> = self.category_index.keys().cloned().collect();
        categories.sort();
        categories
    }

    /// Get all snippets
    pub fn get_all_snippets(&self) -> Vec<&Snippet> {
        let mut snippets: Vec<_> = self.snippets.values().collect();
        snippets.sort_by(|a, b| a.name.cmp(&b.name));
        snippets
    }

    /// Parse a snippet template into parts and placeholders
    pub fn parse_template(&self, template: &str) -> ParsedTemplate {
        let mut parts = Vec::new();
        let mut placeholders = Vec::new();
        let mut final_position = None;
        let mut current_text = String::new();
        let mut chars = template.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '$' {
                // Check for placeholder
                if let Some(&next) = chars.peek() {
                    if next == '{' {
                        // Named placeholder: ${name} or ${name:default}
                        chars.next(); // consume '{'
                        let mut name = String::new();
                        let mut default = String::new();
                        let mut in_default = false;

                        while let Some(&c) = chars.peek() {
                            if c == '}' {
                                chars.next(); // consume '}'
                                break;
                            } else if c == ':' && !in_default {
                                chars.next(); // consume ':'
                                in_default = true;
                            } else {
                                chars.next();
                                if in_default {
                                    default.push(c);
                                } else {
                                    name.push(c);
                                }
                            }
                        }

                        if !current_text.is_empty() {
                            parts.push(TemplatePart::Text(current_text.clone()));
                            current_text.clear();
                        }

                        let placeholder = Placeholder::Named {
                            name,
                            default: if default.is_empty() {
                                None
                            } else {
                                Some(default)
                            },
                        };
                        placeholders.push(placeholder.clone());
                        parts.push(TemplatePart::Placeholder(placeholder));
                    } else if next == '0' {
                        // Final cursor position
                        chars.next(); // consume '0'
                        if !current_text.is_empty() {
                            parts.push(TemplatePart::Text(current_text.clone()));
                            current_text.clear();
                        }
                        let placeholder = Placeholder::Final;
                        final_position = Some(parts.len());
                        parts.push(TemplatePart::Placeholder(placeholder));
                    } else if next.is_ascii_digit() {
                        // Sequential placeholder: $1, $2, etc.
                        let mut num_str = String::new();
                        while let Some(&c) = chars.peek() {
                            if c.is_ascii_digit() {
                                num_str.push(c);
                                chars.next();
                            } else {
                                break;
                            }
                        }

                        if let Ok(num) = num_str.parse::<usize>() {
                            if !current_text.is_empty() {
                                parts.push(TemplatePart::Text(current_text.clone()));
                                current_text.clear();
                            }
                            let placeholder = Placeholder::Sequential(num);
                            placeholders.push(placeholder.clone());
                            parts.push(TemplatePart::Placeholder(placeholder));
                        } else {
                            current_text.push('$');
                            current_text.push_str(&num_str);
                        }
                    } else {
                        // Not a placeholder, just '$'
                        current_text.push(ch);
                    }
                } else {
                    // '$' at end of template
                    current_text.push(ch);
                }
            } else {
                current_text.push(ch);
            }
        }

        if !current_text.is_empty() {
            parts.push(TemplatePart::Text(current_text));
        }

        // Sort placeholders by their order
        placeholders.sort_by(|a, b| match (a, b) {
            (Placeholder::Sequential(a), Placeholder::Sequential(b)) => a.cmp(b),
            (Placeholder::Sequential(_), _) => std::cmp::Ordering::Less,
            (_, Placeholder::Sequential(_)) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        });

        ParsedTemplate {
            parts,
            placeholders,
            final_position,
        }
    }

    /// Expand a snippet template with provided values
    pub fn expand_template(
        &self,
        template: &str,
        values: &HashMap<String, String>,
    ) -> (String, Option<usize>) {
        let parsed = self.parse_template(template);
        let mut result = String::new();
        let mut cursor_offset = None;

        for (idx, part) in parsed.parts.iter().enumerate() {
            match part {
                TemplatePart::Text(text) => result.push_str(text),
                TemplatePart::Placeholder(placeholder) => match placeholder {
                    Placeholder::Sequential(num) => {
                        let key = num.to_string();
                        if let Some(value) = values.get(&key) {
                            result.push_str(value);
                        }
                    }
                    Placeholder::Named { name, default } => {
                        if let Some(value) = values.get(name) {
                            result.push_str(value);
                        } else if let Some(def) = default {
                            result.push_str(def);
                        }
                    }
                    Placeholder::Final => {
                        if Some(idx) == parsed.final_position {
                            cursor_offset = Some(result.len());
                        }
                    }
                },
            }
        }

        // If no $0 was specified, cursor goes to the end
        if cursor_offset.is_none() {
            cursor_offset = Some(result.len());
        }

        (result, cursor_offset)
    }

    /// Save snippets to a JSON file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), SnippetError> {
        let snippets: Vec<&Snippet> = self.snippets.values().collect();
        let json = serde_json::to_string_pretty(&snippets)
            .map_err(|e| SnippetError::SerializationError(e.to_string()))?;
        fs::write(path, json).map_err(|e| SnippetError::IoError(e.to_string()))?;
        Ok(())
    }

    /// Load snippets from a JSON file
    pub fn load_from_file(&mut self, path: &PathBuf) -> Result<(), SnippetError> {
        let json =
            fs::read_to_string(path).map_err(|e| SnippetError::IoError(e.to_string()))?;
        let snippets: Vec<Snippet> = serde_json::from_str(&json)
            .map_err(|e| SnippetError::SerializationError(e.to_string()))?;

        for snippet in snippets {
            self.add_snippet(snippet)?;
        }

        Ok(())
    }

    /// Get default snippets for common use cases
    pub fn with_defaults() -> Self {
        let mut manager = Self::new();

        // Rust snippets
        let _ = manager.add_snippet(
            Snippet::new(
                "Function",
                "Rust function definition",
                "fn",
                "fn ${name}($1) -> $2 {\n    $0\n}",
                "rust",
            )
            .with_tag("function")
            .with_tag("definition"),
        );

        let _ = manager.add_snippet(
            Snippet::new(
                "Test Function",
                "Rust test function",
                "test",
                "#[test]\nfn ${name}() {\n    $0\n}",
                "rust",
            )
            .with_tag("test")
            .with_tag("function"),
        );

        let _ = manager.add_snippet(
            Snippet::new(
                "Struct",
                "Rust struct definition",
                "struct",
                "struct ${name} {\n    $0\n}",
                "rust",
            )
            .with_tag("struct")
            .with_tag("definition"),
        );

        let _ = manager.add_snippet(
            Snippet::new(
                "Impl Block",
                "Rust implementation block",
                "impl",
                "impl ${name} {\n    $0\n}",
                "rust",
            )
            .with_tag("impl")
            .with_tag("implementation"),
        );

        let _ = manager.add_snippet(
            Snippet::new(
                "Match Expression",
                "Rust match expression",
                "match",
                "match ${expr} {\n    ${pattern} => $0,\n}",
                "rust",
            )
            .with_tag("match")
            .with_tag("control-flow"),
        );

        // Bash snippets
        let _ = manager.add_snippet(
            Snippet::new(
                "If Statement",
                "Bash if statement",
                "if",
                "if [ ${condition} ]; then\n    $0\nfi",
                "bash",
            )
            .with_tag("if")
            .with_tag("control-flow"),
        );

        let _ = manager.add_snippet(
            Snippet::new(
                "For Loop",
                "Bash for loop",
                "for",
                "for ${var} in ${list}; do\n    $0\ndone",
                "bash",
            )
            .with_tag("for")
            .with_tag("loop"),
        );

        let _ = manager.add_snippet(
            Snippet::new(
                "Function",
                "Bash function definition",
                "func",
                "${name}() {\n    $0\n}",
                "bash",
            )
            .with_tag("function")
            .with_tag("definition"),
        );

        // Git snippets
        let _ = manager.add_snippet(
            Snippet::new(
                "Git Commit",
                "Git commit with message",
                "gc",
                "git commit -m \"${message}\"$0",
                "git",
            )
            .with_tag("commit")
            .with_tag("git"),
        );

        let _ = manager.add_snippet(
            Snippet::new(
                "Git Branch",
                "Create and checkout new branch",
                "gb",
                "git checkout -b ${branch-name}$0",
                "git",
            )
            .with_tag("branch")
            .with_tag("git"),
        );

        let _ = manager.add_snippet(
            Snippet::new(
                "Git Push",
                "Push to remote",
                "gp",
                "git push origin ${branch:main}$0",
                "git",
            )
            .with_tag("push")
            .with_tag("git"),
        );

        // Docker snippets
        let _ = manager.add_snippet(
            Snippet::new(
                "Docker Run",
                "Run a Docker container",
                "drun",
                "docker run -it --rm ${image} $0",
                "docker",
            )
            .with_tag("docker")
            .with_tag("run"),
        );

        let _ = manager.add_snippet(
            Snippet::new(
                "Docker Compose",
                "Docker compose up",
                "dup",
                "docker-compose up -d $0",
                "docker",
            )
            .with_tag("docker")
            .with_tag("compose"),
        );

        manager
    }
}

impl Default for SnippetManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during snippet operations
#[derive(Debug, thiserror::Error)]
pub enum SnippetError {
    #[error("Snippet not found: {0}")]
    NotFound(String),
    #[error("Duplicate trigger: {0}")]
    DuplicateTrigger(String),
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snippet_creation() {
        let snippet = Snippet::new(
            "Test Snippet",
            "A test snippet",
            "test",
            "template $1 $2",
            "test-category",
        );

        assert_eq!(snippet.name, "Test Snippet");
        assert_eq!(snippet.description, "A test snippet");
        assert_eq!(snippet.trigger, "test");
        assert_eq!(snippet.template, "template $1 $2");
        assert_eq!(snippet.category, "test-category");
    }

    #[test]
    fn test_snippet_with_tags() {
        let snippet = Snippet::new("Test", "desc", "test", "template", "cat")
            .with_tag("tag1")
            .with_tag("tag2");

        assert_eq!(snippet.tags, vec!["tag1", "tag2"]);
    }

    #[test]
    fn test_snippet_manager_add() {
        let mut manager = SnippetManager::new();
        let snippet = Snippet::new("Test", "desc", "test", "template", "cat");

        assert!(manager.add_snippet(snippet.clone()).is_ok());
        assert!(manager.get_snippet(&snippet.id).is_some());
    }

    #[test]
    fn test_duplicate_trigger() {
        let mut manager = SnippetManager::new();
        let snippet1 = Snippet::new("Test1", "desc", "test", "template", "cat");
        let snippet2 = Snippet::new("Test2", "desc", "test", "template", "cat");

        assert!(manager.add_snippet(snippet1).is_ok());
        assert!(matches!(
            manager.add_snippet(snippet2),
            Err(SnippetError::DuplicateTrigger(_))
        ));
    }

    #[test]
    fn test_remove_snippet() {
        let mut manager = SnippetManager::new();
        let snippet = Snippet::new("Test", "desc", "test", "template", "cat");
        let id = snippet.id.clone();

        manager.add_snippet(snippet).unwrap();
        assert!(manager.get_snippet(&id).is_some());

        let removed = manager.remove_snippet(&id);
        assert!(removed.is_ok());
        assert!(manager.get_snippet(&id).is_none());
    }

    #[test]
    fn test_find_by_trigger() {
        let mut manager = SnippetManager::new();
        manager
            .add_snippet(Snippet::new("Test1", "desc", "test1", "template", "cat"))
            .unwrap();
        manager
            .add_snippet(Snippet::new("Test2", "desc", "test2", "template", "cat"))
            .unwrap();
        manager
            .add_snippet(Snippet::new("Other", "desc", "other", "template", "cat"))
            .unwrap();

        let results = manager.find_by_trigger("test");
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|s| s.trigger.starts_with("test")));
    }

    #[test]
    fn test_find_exact_trigger() {
        let mut manager = SnippetManager::new();
        manager
            .add_snippet(Snippet::new("Test", "desc", "test", "template", "cat"))
            .unwrap();

        let result = manager.find_exact_trigger("test");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "Test");

        let not_found = manager.find_exact_trigger("notfound");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_get_by_category() {
        let mut manager = SnippetManager::new();
        manager
            .add_snippet(Snippet::new("Test1", "desc", "t1", "template", "rust"))
            .unwrap();
        manager
            .add_snippet(Snippet::new("Test2", "desc", "t2", "template", "rust"))
            .unwrap();
        manager
            .add_snippet(Snippet::new("Test3", "desc", "t3", "template", "bash"))
            .unwrap();

        let rust_snippets = manager.get_by_category("rust");
        assert_eq!(rust_snippets.len(), 2);

        let bash_snippets = manager.get_by_category("bash");
        assert_eq!(bash_snippets.len(), 1);
    }

    #[test]
    fn test_parse_sequential_placeholders() {
        let manager = SnippetManager::new();
        let parsed = manager.parse_template("Hello $1, welcome to $2!");

        assert_eq!(parsed.placeholders.len(), 2);
        assert_eq!(parsed.placeholders[0], Placeholder::Sequential(1));
        assert_eq!(parsed.placeholders[1], Placeholder::Sequential(2));
        assert_eq!(parsed.parts.len(), 5); // text, ph, text, ph, text
    }

    #[test]
    fn test_parse_named_placeholders() {
        let manager = SnippetManager::new();
        let parsed = manager.parse_template("fn ${name}() -> ${type:Result} {}");

        assert_eq!(parsed.placeholders.len(), 2);
        match &parsed.placeholders[0] {
            Placeholder::Named { name, default } => {
                assert_eq!(name, "name");
                assert_eq!(default, &None);
            }
            _ => panic!("Expected named placeholder"),
        }
        match &parsed.placeholders[1] {
            Placeholder::Named { name, default } => {
                assert_eq!(name, "type");
                assert_eq!(default, &Some("Result".to_string()));
            }
            _ => panic!("Expected named placeholder"),
        }
    }

    #[test]
    fn test_parse_final_placeholder() {
        let manager = SnippetManager::new();
        let parsed = manager.parse_template("Start $1 middle $0 end");

        assert_eq!(parsed.placeholders.len(), 1);
        assert!(parsed.final_position.is_some());
    }

    #[test]
    fn test_expand_template_sequential() {
        let manager = SnippetManager::new();
        let mut values = HashMap::new();
        values.insert("1".to_string(), "World".to_string());
        values.insert("2".to_string(), "AgTerm".to_string());

        let (result, _) = manager.expand_template("Hello $1, welcome to $2!", &values);
        assert_eq!(result, "Hello World, welcome to AgTerm!");
    }

    #[test]
    fn test_expand_template_named() {
        let manager = SnippetManager::new();
        let mut values = HashMap::new();
        values.insert("name".to_string(), "my_function".to_string());

        let (result, _) = manager.expand_template("fn ${name}() -> ${type:Result} {}", &values);
        assert_eq!(result, "fn my_function() -> Result {}");
    }

    #[test]
    fn test_expand_template_final_position() {
        let manager = SnippetManager::new();
        let values = HashMap::new();

        let (result, cursor) = manager.expand_template("Start $0 end", &values);
        assert_eq!(result, "Start  end");
        assert_eq!(cursor, Some(6)); // Position after "Start "
    }

    #[test]
    fn test_expand_template_no_final_position() {
        let manager = SnippetManager::new();
        let values = HashMap::new();

        let (result, cursor) = manager.expand_template("Hello World", &values);
        assert_eq!(result, "Hello World");
        assert_eq!(cursor, Some(11)); // End of string
    }

    #[test]
    fn test_update_snippet() {
        let mut manager = SnippetManager::new();
        let snippet = Snippet::new("Original", "desc", "test", "template", "cat");
        let id = snippet.id.clone();

        manager.add_snippet(snippet).unwrap();

        let updated = Snippet::new("Updated", "new desc", "test2", "new template", "cat2");
        manager.update_snippet(&id, updated).unwrap();

        let retrieved = manager.get_snippet(&id).unwrap();
        assert_eq!(retrieved.name, "Updated");
        assert_eq!(retrieved.trigger, "test2");
    }

    #[test]
    fn test_default_snippets() {
        let manager = SnippetManager::with_defaults();

        // Check that we have snippets
        assert!(!manager.get_all_snippets().is_empty());

        // Check for some expected categories
        let categories = manager.get_categories();
        assert!(categories.contains(&"rust".to_string()));
        assert!(categories.contains(&"bash".to_string()));
        assert!(categories.contains(&"git".to_string()));

        // Test that we can find by trigger
        let fn_snippet = manager.find_exact_trigger("fn");
        assert!(fn_snippet.is_some());
        assert_eq!(fn_snippet.unwrap().category, "rust");
    }

    #[test]
    fn test_complex_template() {
        let manager = SnippetManager::new();
        let template = "fn ${name}($1) -> ${return:Result<(), Error>} {\n    $2\n    $0\n}";

        let mut values = HashMap::new();
        values.insert("name".to_string(), "process".to_string());
        values.insert("1".to_string(), "data: &str".to_string());
        values.insert("2".to_string(), "println!(\"Processing: {}\", data);".to_string());

        let (result, cursor) = manager.expand_template(template, &values);

        let expected = "fn process(data: &str) -> Result<(), Error> {\n    println!(\"Processing: {}\", data);\n    \n}";
        assert_eq!(result, expected);
        assert!(cursor.is_some());
    }

    #[test]
    fn test_serialization() {
        let mut manager = SnippetManager::new();
        manager
            .add_snippet(Snippet::new("Test", "desc", "test", "template", "cat"))
            .unwrap();

        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("snippets.json");

        // Save
        assert!(manager.save_to_file(&file_path).is_ok());

        // Load into new manager
        let mut new_manager = SnippetManager::new();
        assert!(new_manager.load_from_file(&file_path).is_ok());

        // Verify
        let snippets = new_manager.get_all_snippets();
        assert_eq!(snippets.len(), 1);
        assert_eq!(snippets[0].name, "Test");
    }

    #[test]
    fn test_edge_case_empty_template() {
        let manager = SnippetManager::new();
        let parsed = manager.parse_template("");
        assert!(parsed.parts.is_empty());
        assert!(parsed.placeholders.is_empty());
    }

    #[test]
    fn test_edge_case_dollar_at_end() {
        let manager = SnippetManager::new();
        let parsed = manager.parse_template("Price: $");
        assert_eq!(parsed.parts.len(), 1);
        match &parsed.parts[0] {
            TemplatePart::Text(text) => assert_eq!(text, "Price: $"),
            _ => panic!("Expected text part"),
        }
    }

    #[test]
    fn test_edge_case_malformed_placeholder() {
        let manager = SnippetManager::new();
        let parsed = manager.parse_template("${incomplete");
        // Should treat as text since it's not properly closed
        assert_eq!(parsed.placeholders.len(), 1);
    }
}

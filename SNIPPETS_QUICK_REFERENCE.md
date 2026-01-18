# Snippet System Quick Reference

## Import

```rust
use agterm::snippets::{Snippet, SnippetManager};
use std::collections::HashMap;
```

## Create Manager

```rust
// Empty manager
let mut manager = SnippetManager::new();

// With default snippets (rust, bash, git, docker)
let manager = SnippetManager::with_defaults();
```

## Create Snippet

```rust
let snippet = Snippet::new(
    "name",        // Display name
    "description", // What it does
    "trigger",     // Abbreviation
    "template",    // Template with placeholders
    "category",    // Group (rust, bash, etc.)
);

// Add tags
let snippet = snippet
    .with_tag("tag1")
    .with_tag("tag2");
```

## Add/Remove Snippets

```rust
// Add
manager.add_snippet(snippet)?;

// Remove
let removed = manager.remove_snippet(&id)?;

// Update
manager.update_snippet(&id, new_snippet)?;
```

## Find Snippets

```rust
// Exact trigger
let snippet = manager.find_exact_trigger("fn");

// Prefix search (for autocomplete)
let matches = manager.find_by_trigger("te"); // test, test2, ...

// By category
let rust_snippets = manager.get_by_category("rust");

// All snippets
let all = manager.get_all_snippets();

// All categories
let categories = manager.get_categories();
```

## Placeholder Syntax

```rust
// Sequential: $1, $2, $3
"Hello $1, welcome to $2!"

// Named: ${name}
"fn ${name}() {}"

// Named with default: ${name:default}
"fn ${name}() -> ${type:Result<()>} {}"

// Final cursor position: $0
"for ${var} in ${list} {\n    $0\n}"
```

## Expand Template

```rust
let mut values = HashMap::new();
values.insert("name".to_string(), "my_func".to_string());
values.insert("1".to_string(), "arg: i32".to_string());

let (expanded_text, cursor_position) =
    manager.expand_template(&snippet.template, &values);
```

## Parse Template

```rust
let parsed = manager.parse_template("fn ${name}($1) -> $2 {}");

// Access components
for part in parsed.parts {
    match part {
        TemplatePart::Text(text) => println!("Text: {}", text),
        TemplatePart::Placeholder(ph) => println!("Placeholder: {:?}", ph),
    }
}

println!("Placeholders: {:?}", parsed.placeholders);
println!("Has $0: {}", parsed.final_position.is_some());
```

## Save/Load

```rust
use std::path::PathBuf;

// Save
let path = PathBuf::from("snippets.json");
manager.save_to_file(&path)?;

// Load
manager.load_from_file(&path)?;
```

## Error Handling

```rust
use agterm::snippets::SnippetError;

match manager.add_snippet(snippet) {
    Ok(_) => {},
    Err(SnippetError::DuplicateTrigger(t)) => {
        eprintln!("Trigger '{}' already exists", t);
    },
    Err(e) => eprintln!("Error: {}", e),
}
```

## Common Patterns

### Autocomplete Integration

```rust
// User types prefix
let prefix = "te";
let matches = manager.find_by_trigger(prefix);

// Show suggestions
for snippet in matches {
    println!("{} - {}", snippet.trigger, snippet.name);
}

// User selects one
if let Some(snippet) = manager.find_exact_trigger("test") {
    // Collect placeholder values from user
    let mut values = HashMap::new();
    values.insert("name".to_string(), user_input);

    // Expand and insert
    let (text, cursor) = manager.expand_template(
        &snippet.template,
        &values
    );

    insert_text(text);
    move_cursor(cursor.unwrap_or(text.len()));
}
```

### Building Custom Snippet Library

```rust
let mut manager = SnippetManager::new();

// Add project-specific snippets
manager.add_snippet(Snippet::new(
    "HTTP Handler",
    "Async HTTP handler",
    "handler",
    r#"async fn ${name}(
    State(state): State<AppState>,
) -> Result<Json<Response>> {
    $0
}"#,
    "web",
).with_tag("async").with_tag("api"))?;

// Save for later
manager.save_to_file(&PathBuf::from("my_snippets.json"))?;
```

## Default Snippets

| Trigger | Category | Description |
|---------|----------|-------------|
| `fn` | rust | Function definition |
| `test` | rust | Test function |
| `struct` | rust | Struct definition |
| `impl` | rust | Implementation block |
| `match` | rust | Match expression |
| `if` | bash | If statement |
| `for` | bash | For loop |
| `func` | bash | Function definition |
| `gc` | git | Git commit |
| `gb` | git | Git branch |
| `gp` | git | Git push |
| `drun` | docker | Docker run |
| `dup` | docker | Docker compose up |

## Examples

### Simple Template

```rust
// Template
"Hello $1, you are $2 years old!"

// Values
{"1": "Alice", "2": "25"}

// Result
"Hello Alice, you are 25 years old!"
```

### Named with Defaults

```rust
// Template
"fn ${name}() -> ${type:Result<()>} {}"

// Values (only name provided)
{"name": "process"}

// Result (uses default for type)
"fn process() -> Result<()> {}"
```

### Complex Multi-line

```rust
// Template
r#"fn ${name}($1) -> ${return:Result<()>} {
    // TODO: Implement ${name}
    $2
    $0
}"#

// Values
{
    "name": "process",
    "1": "data: &str",
    "2": "println!(\"Processing...\");"
}

// Result (cursor at $0)
r#"fn process(data: &str) -> Result<()> {
    // TODO: Implement process
    println!("Processing...");
    █  // Cursor here
}"#
```

## Testing

```bash
# Run all snippet tests
cargo test --lib snippets

# Run integration tests
cargo test --test snippets_integration_test

# Run demo
cargo run --example snippets_demo
```

## Performance

- Add: O(1) average
- Remove: O(1) average
- Find by trigger: O(k) where k = matching triggers
- Get by category: O(m) where m = snippets in category
- Parse: O(n) where n = template length
- Expand: O(n + p) where p = placeholders

## Tips

1. Keep triggers short (2-5 chars)
2. Use descriptive names
3. Provide defaults for common cases
4. Always include $0 for cursor position
5. Group related snippets by category
6. Use tags for better searchability
7. Test templates before saving

# AgTerm Snippet System

A powerful code snippet system with template expansion and placeholder support for AgTerm terminal.

## Overview

The snippet system provides:
- **Snippet Management**: Create, read, update, and delete code snippets
- **Template Expansion**: Support for sequential ($1, $2) and named (${name:default}) placeholders
- **Final Cursor Position**: Use $0 to specify where the cursor should end up
- **Category Organization**: Group snippets by language or use case
- **Trigger-based Autocomplete**: Fast lookup by abbreviation
- **Persistence**: Save and load snippets from JSON files

## Quick Start

```rust
use agterm::snippets::{Snippet, SnippetManager};
use std::collections::HashMap;

// Create a manager with default snippets
let manager = SnippetManager::with_defaults();

// Find a snippet by trigger
let snippet = manager.find_exact_trigger("fn").unwrap();

// Expand the template
let mut values = HashMap::new();
values.insert("name".to_string(), "my_function".to_string());
values.insert("1".to_string(), "arg: i32".to_string());
values.insert("2".to_string(), "i32".to_string());

let (expanded, cursor_pos) = manager.expand_template(&snippet.template, &values);
// Result: "fn my_function(arg: i32) -> i32 {\n    \n}"
```

## Placeholder Syntax

### Sequential Placeholders

Use `$1`, `$2`, `$3`, etc. for simple ordered placeholders:

```
Template: "Hello $1, welcome to $2!"
Values: {"1": "Alice", "2": "AgTerm"}
Result: "Hello Alice, welcome to AgTerm!"
```

### Named Placeholders

Use `${name}` for descriptive placeholders:

```
Template: "fn ${name}(${args}) -> ${return} {}"
Values: {"name": "process", "args": "data: &str", "return": "Result<()>"}
Result: "fn process(data: &str) -> Result<()> {}"
```

### Named Placeholders with Defaults

Use `${name:default}` to provide fallback values:

```
Template: "fn ${name}() -> ${type:Result<(), Error>} {}"
Values: {"name": "run"}  // 'type' not provided
Result: "fn run() -> Result<(), Error> {}"
```

### Final Cursor Position

Use `$0` to specify where the cursor should be placed after expansion:

```
Template: "for ${var} in ${list} {\n    $0\n}"
Result: Cursor positioned at the indented line inside the loop
```

## Creating Snippets

### Basic Snippet

```rust
let snippet = Snippet::new(
    "Function",                          // name
    "Rust function definition",          // description
    "fn",                                // trigger
    "fn ${name}($1) -> $2 {\n    $0\n}", // template
    "rust",                              // category
);
```

### Snippet with Tags

```rust
let snippet = Snippet::new(
    "Test Function",
    "Rust test function",
    "test",
    "#[test]\nfn ${name}() {\n    $0\n}",
    "rust",
)
.with_tag("test")
.with_tag("function")
.with_tag("unit-test");
```

## Using the Snippet Manager

### Adding Snippets

```rust
let mut manager = SnippetManager::new();
let snippet = Snippet::new(...);

manager.add_snippet(snippet)?;
```

### Finding Snippets

```rust
// Exact trigger match
let snippet = manager.find_exact_trigger("fn");

// Prefix search (for autocomplete)
let matches = manager.find_by_trigger("tes"); // Returns "test", "test2", etc.

// By category
let rust_snippets = manager.get_by_category("rust");

// All snippets
let all = manager.get_all_snippets();
```

### Updating Snippets

```rust
let updated = Snippet::new(
    "Updated Function",
    "Updated description",
    "fn2",
    "new template",
    "rust",
);

manager.update_snippet(&snippet_id, updated)?;
```

### Removing Snippets

```rust
let removed = manager.remove_snippet(&snippet_id)?;
```

## Template Parsing

Parse templates to inspect their structure:

```rust
let parsed = manager.parse_template("fn ${name}($1) -> $2 {\n    $0\n}");

// Access parsed components
for part in parsed.parts {
    match part {
        TemplatePart::Text(text) => println!("Text: {}", text),
        TemplatePart::Placeholder(ph) => println!("Placeholder: {:?}", ph),
    }
}

// Check placeholders
println!("Placeholders: {:?}", parsed.placeholders);
println!("Has final position: {}", parsed.final_position.is_some());
```

## Persistence

### Saving to File

```rust
use std::path::PathBuf;

let path = PathBuf::from("snippets.json");
manager.save_to_file(&path)?;
```

### Loading from File

```rust
let mut manager = SnippetManager::new();
manager.load_from_file(&path)?;
```

## Default Snippets

The system comes with built-in snippets for common use cases:

### Rust

- `fn` - Function definition
- `test` - Test function
- `struct` - Struct definition
- `impl` - Implementation block
- `match` - Match expression

### Bash

- `if` - If statement
- `for` - For loop
- `func` - Function definition

### Git

- `gc` - Git commit
- `gb` - Git branch
- `gp` - Git push

### Docker

- `drun` - Docker run
- `dup` - Docker compose up

## Advanced Examples

### Complex Template with Multiple Placeholder Types

```rust
let template = r#"
fn ${name}($1) -> ${return:Result<(), Error>} {
    // TODO: Implement ${name}
    $2
    $0
}
"#;

let mut values = HashMap::new();
values.insert("name".to_string(), "process_data".to_string());
values.insert("1".to_string(), "input: &str".to_string());
values.insert("2".to_string(), "println!(\"Processing: {}\", input);".to_string());
// 'return' not provided, will use default "Result<(), Error>"

let (expanded, cursor_pos) = manager.expand_template(template, &values);
```

Result:
```rust
fn process_data(input: &str) -> Result<(), Error> {
    // TODO: Implement process_data
    println!("Processing: {}", input);
    █  // Cursor here (marked as $0)
}
```

### Snippet with Newlines and Indentation

```rust
let snippet = Snippet::new(
    "Match Expression",
    "Rust match with multiple arms",
    "match",
    r#"match ${expr} {
    ${pattern1} => ${action1},
    ${pattern2} => ${action2},
    _ => $0,
}"#,
    "rust",
);
```

### Building a Snippet Library

```rust
// Create a custom snippet manager
let mut manager = SnippetManager::new();

// Add project-specific snippets
manager.add_snippet(Snippet::new(
    "API Handler",
    "HTTP handler template",
    "handler",
    r#"async fn ${name}_handler(
    State(state): State<AppState>,
    Json(payload): Json<${Request}>,
) -> Result<Json<${Response}>, ${Error}> {
    $0
}"#,
    "api",
).with_tag("async").with_tag("handler"))?;

// Save for reuse
manager.save_to_file(&PathBuf::from("my_snippets.json"))?;
```

## Integration with Autocomplete

The snippet system is designed to integrate with AgTerm's completion engine:

```rust
// When user types a trigger prefix
let prefix = "te";
let matching_snippets = manager.find_by_trigger(prefix);

// Convert to completion items
for snippet in matching_snippets {
    let completion = CompletionItem {
        text: snippet.trigger.clone(),
        kind: CompletionKind::Snippet,
        description: Some(snippet.name.clone()),
    };
    // Add to completion list
}

// On completion selection
if let Some(snippet) = manager.find_exact_trigger(&selected_trigger) {
    // Show placeholder input UI
    // Collect values from user
    let (expanded, cursor_pos) = manager.expand_template(
        &snippet.template,
        &collected_values
    );
    // Insert expanded text and move cursor
}
```

## Error Handling

The snippet system uses the `SnippetError` enum for error handling:

```rust
use agterm::snippets::SnippetError;

match manager.add_snippet(snippet) {
    Ok(_) => println!("Snippet added successfully"),
    Err(SnippetError::DuplicateTrigger(trigger)) => {
        eprintln!("Trigger '{}' already exists", trigger);
    },
    Err(e) => eprintln!("Error: {}", e),
}
```

Error types:
- `NotFound(String)` - Snippet ID not found
- `DuplicateTrigger(String)` - Trigger already in use
- `IoError(String)` - File I/O error
- `SerializationError(String)` - JSON serialization error

## Best Practices

1. **Use Descriptive Names**: Make snippet names clear and searchable
2. **Short Triggers**: Keep triggers short and memorable (2-5 characters)
3. **Category Organization**: Group related snippets in categories
4. **Default Values**: Provide sensible defaults for common cases
5. **Cursor Position**: Always use $0 to guide the user where to type next
6. **Consistent Indentation**: Match the indentation style of the target language
7. **Tag Liberally**: Use tags to make snippets easier to find

## Testing

The snippet system includes comprehensive tests:

```bash
# Run all snippet tests
cargo test --lib snippets

# Run with output
cargo test --lib snippets -- --nocapture

# Run specific test
cargo test --lib test_expand_template_named
```

## Running the Demo

```bash
cargo run --example snippets_demo
```

This will demonstrate:
- Loading default snippets
- Template expansion with different placeholder types
- Creating custom snippets
- Category-based organization
- Template parsing

## Future Enhancements

Potential improvements for the snippet system:

- **Variables**: Support for built-in variables like `$DATE`, `$USER`, `$CLIPBOARD`
- **Transformations**: Apply transformations to placeholder values (uppercase, lowercase, etc.)
- **Conditional Content**: Show/hide template sections based on values
- **Nested Placeholders**: Support placeholders within placeholder defaults
- **Multi-line Placeholders**: Allow placeholders to span multiple lines
- **Snippet Inheritance**: Create snippets that extend other snippets
- **Import/Export**: Share snippet collections between users

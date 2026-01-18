# Snippet System Architecture

## Overview Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        Snippet System                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌────────────────────────────────────────────────────────┐    │
│  │                   SnippetManager                       │    │
│  │                                                         │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐│    │
│  │  │   snippets   │  │trigger_index │  │category_index││    │
│  │  │ HashMap<ID,  │  │HashMap<Trig, │  │HashMap<Cat,  ││    │
│  │  │  Snippet>    │  │  Vec<ID>>    │  │  Vec<ID>>    ││    │
│  │  └──────────────┘  └──────────────┘  └──────────────┘│    │
│  │                                                         │    │
│  │  CRUD Operations:                                      │    │
│  │  • add_snippet()                                       │    │
│  │  • remove_snippet()                                    │    │
│  │  • update_snippet()                                    │    │
│  │  • get_snippet()                                       │    │
│  │                                                         │    │
│  │  Search Operations:                                    │    │
│  │  • find_by_trigger()        ← O(k) prefix search      │    │
│  │  • find_exact_trigger()     ← O(1) exact match        │    │
│  │  • get_by_category()        ← O(1) category lookup    │    │
│  │  • get_all_snippets()       ← O(n) all snippets       │    │
│  │                                                         │    │
│  │  Template Operations:                                  │    │
│  │  • parse_template()         ← Parse placeholders      │    │
│  │  • expand_template()        ← Substitute values       │    │
│  │                                                         │    │
│  │  Persistence:                                          │    │
│  │  • save_to_file()           ← JSON export             │    │
│  │  • load_from_file()         ← JSON import             │    │
│  └────────────────────────────────────────────────────────┘    │
│                                                                   │
│  ┌────────────────────────────────────────────────────────┐    │
│  │                      Snippet                           │    │
│  │                                                         │    │
│  │  • id: String              ← UUID                      │    │
│  │  • name: String            ← Display name              │    │
│  │  • description: String     ← What it does              │    │
│  │  • trigger: String         ← Abbreviation              │    │
│  │  • template: String        ← Content with placeholders │    │
│  │  • category: String        ← Group (rust, bash, etc.)  │    │
│  │  • tags: Vec<String>       ← Additional metadata       │    │
│  └────────────────────────────────────────────────────────┘    │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

## Data Flow

### Adding a Snippet

```
User Input
   │
   ├─→ Create Snippet
   │      │
   │      └─→ Generate UUID
   │
   └─→ SnippetManager.add_snippet()
          │
          ├─→ Check for duplicate trigger
          │      │
          │      └─→ Return DuplicateTrigger error if exists
          │
          ├─→ Add to snippets HashMap
          │
          ├─→ Update trigger_index
          │
          └─→ Update category_index
```

### Finding and Expanding a Snippet

```
User Types "te" + Tab
   │
   └─→ SnippetManager.find_by_trigger("te")
          │
          └─→ Iterate trigger_index, find prefixes
                 │
                 └─→ Return matching snippets

User Selects "test"
   │
   └─→ SnippetManager.find_exact_trigger("test")
          │
          └─→ Return snippet

Show Placeholder Input UI
   │
   └─→ Collect values from user
          │
          └─→ SnippetManager.expand_template(template, values)
                 │
                 ├─→ Parse template into parts
                 │      │
                 │      ├─→ Identify $1, $2, ... (sequential)
                 │      ├─→ Identify ${name} (named)
                 │      ├─→ Identify ${name:default} (named with default)
                 │      └─→ Identify $0 (final position)
                 │
                 ├─→ Substitute placeholders with values
                 │      │
                 │      └─→ Use defaults if value not provided
                 │
                 └─→ Return (expanded_text, cursor_position)

Insert text and move cursor
```

## Placeholder Types

```
┌─────────────────────────────────────────────────────────────┐
│                    Placeholder Types                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Sequential: $N                                            │
│  ┌──────────────────────────────────────────┐             │
│  │  $1, $2, $3, ... $N                       │             │
│  │                                            │             │
│  │  Example: "Hello $1, you are $2!"         │             │
│  │  Values:  {"1": "Alice", "2": "25"}       │             │
│  │  Result:  "Hello Alice, you are 25!"      │             │
│  └──────────────────────────────────────────┘             │
│                                                             │
│  Named: ${name}                                            │
│  ┌──────────────────────────────────────────┐             │
│  │  ${identifier}                             │             │
│  │                                            │             │
│  │  Example: "fn ${name}() {}"                │             │
│  │  Values:  {"name": "process"}              │             │
│  │  Result:  "fn process() {}"                │             │
│  └──────────────────────────────────────────┘             │
│                                                             │
│  Named with Default: ${name:default}                       │
│  ┌──────────────────────────────────────────┐             │
│  │  ${identifier:default_value}               │             │
│  │                                            │             │
│  │  Example: "fn ${n}() -> ${t:Result<()>}"  │             │
│  │  Values:  {"n": "main"}  (no "t")         │             │
│  │  Result:  "fn main() -> Result<()>"       │             │
│  └──────────────────────────────────────────┘             │
│                                                             │
│  Final Cursor: $0                                          │
│  ┌──────────────────────────────────────────┐             │
│  │  $0 (marks cursor position)                │             │
│  │                                            │             │
│  │  Example: "for x in y {\n    $0\n}"       │             │
│  │  Result:  Cursor positioned at indent     │             │
│  └──────────────────────────────────────────┘             │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Template Parsing State Machine

```
Input: "fn ${name}($1) -> $2 { $0 }"

State Machine:
┌──────┐
│ TEXT │ ──── 'f', 'n', ' ' ───→ Text buffer: "fn "
└──────┘
   │
   └─ '$' ─→ ┌───────────┐
              │ DOLLAR    │
              └───────────┘
                   │
                   ├─ '{' ─→ ┌─────────────┐
                   │          │ NAMED_START │
                   │          └─────────────┘
                   │               │
                   │               ├─ letters ─→ name buffer
                   │               ├─ ':' ─────→ default buffer
                   │               └─ '}' ─────→ Create Named placeholder
                   │
                   ├─ '0' ─→ Create Final placeholder
                   │
                   └─ digit ─→ ┌──────────────┐
                               │ SEQUENTIAL   │
                               └──────────────┘
                                    │
                                    └─ more digits ─→ Create Sequential placeholder

Result:
  parts: [
    Text("fn "),
    Placeholder(Named { name: "name", default: None }),
    Text("("),
    Placeholder(Sequential(1)),
    Text(") -> "),
    Placeholder(Sequential(2)),
    Text(" { "),
    Placeholder(Final),
    Text(" }"),
  ]
```

## Index Structure

```
┌─────────────────────────────────────────────────────────────┐
│                    Dual Index System                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  trigger_index: HashMap<String, Vec<String>>               │
│  ┌──────────────────────────────────────────┐             │
│  │  "fn"    → ["id-1"]                       │             │
│  │  "test"  → ["id-2"]                       │             │
│  │  "test2" → ["id-3"]                       │             │
│  │  "impl"  → ["id-4"]                       │             │
│  │  ...                                       │             │
│  └──────────────────────────────────────────┘             │
│       ↓                                                     │
│  Fast O(1) lookup by exact trigger                         │
│  Fast O(k) prefix search for autocomplete                  │
│                                                             │
│  category_index: HashMap<String, Vec<String>>              │
│  ┌──────────────────────────────────────────┐             │
│  │  "rust"   → ["id-1", "id-2", "id-3"]     │             │
│  │  "bash"   → ["id-5", "id-6"]             │             │
│  │  "git"    → ["id-7", "id-8", "id-9"]     │             │
│  │  "docker" → ["id-10"]                    │             │
│  │  ...                                       │             │
│  └──────────────────────────────────────────┘             │
│       ↓                                                     │
│  Fast O(1) lookup by category                              │
│  Easy grouping and filtering                               │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Integration Points

```
┌──────────────────────────────────────────────────────────────┐
│                  AgTerm Integration                          │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  Terminal Input                                             │
│      │                                                       │
│      ├─→ Detect trigger pattern                            │
│      │      │                                                │
│      │      └─→ Query SnippetManager.find_by_trigger()     │
│      │             │                                         │
│      │             └─→ Show autocomplete dropdown           │
│      │                                                       │
│      └─→ On selection                                       │
│             │                                                │
│             ├─→ Parse template                              │
│             │      │                                         │
│             │      └─→ Identify placeholders                │
│             │                                                │
│             ├─→ Show placeholder input UI                   │
│             │      │                                         │
│             │      └─→ Tab between fields                   │
│             │                                                │
│             ├─→ Expand template with values                 │
│             │                                                │
│             └─→ Insert text and move cursor                 │
│                                                              │
│  Configuration System                                        │
│      │                                                       │
│      ├─→ Load snippets from ~/.config/agterm/snippets.json │
│      │                                                       │
│      └─→ Auto-save on changes                              │
│                                                              │
│  UI Components                                              │
│      │                                                       │
│      ├─→ Snippet Manager Window                            │
│      │      ├─→ List all snippets                          │
│      │      ├─→ Filter by category                         │
│      │      ├─→ Edit/Delete snippets                       │
│      │      └─→ Create new snippets                        │
│      │                                                       │
│      └─→ Snippet Browser (Ctrl+Shift+S)                    │
│             ├─→ Search by name/trigger                     │
│             ├─→ Preview template                            │
│             └─→ Quick insert                                │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

## Class Diagram

```
┌─────────────────────────┐
│      Snippet            │
├─────────────────────────┤
│ - id: String            │
│ - name: String          │
│ - description: String   │
│ - trigger: String       │
│ - template: String      │
│ - category: String      │
│ - tags: Vec<String>     │
├─────────────────────────┤
│ + new()                 │
│ + with_tag()            │
│ + with_tags()           │
└─────────────────────────┘
           △
           │ contains
           │
┌────────────────────────────────────────┐
│         SnippetManager                 │
├────────────────────────────────────────┤
│ - snippets: HashMap<String, Snippet>   │
│ - trigger_index: HashMap<String, Vec>  │
│ - category_index: HashMap<String, Vec> │
├────────────────────────────────────────┤
│ + new()                                │
│ + with_defaults()                      │
│ + add_snippet()                        │
│ + remove_snippet()                     │
│ + update_snippet()                     │
│ + get_snippet()                        │
│ + find_by_trigger()                    │
│ + find_exact_trigger()                 │
│ + get_by_category()                    │
│ + get_categories()                     │
│ + get_all_snippets()                   │
│ + parse_template()                     │
│ + expand_template()                    │
│ + save_to_file()                       │
│ + load_from_file()                     │
└────────────────────────────────────────┘
           │
           │ produces
           ↓
┌────────────────────────┐
│   ParsedTemplate       │
├────────────────────────┤
│ - parts: Vec           │
│ - placeholders: Vec    │
│ - final_position: Opt  │
└────────────────────────┘
           │
           │ contains
           ↓
┌─────────────────────────┐
│      TemplatePart       │
├─────────────────────────┤
│ Text(String)            │
│ Placeholder(Placeholder)│
└─────────────────────────┘
           │
           │ uses
           ↓
┌──────────────────────────────┐
│       Placeholder            │
├──────────────────────────────┤
│ Sequential(usize)            │
│ Named { name, default }      │
│ Final                        │
└──────────────────────────────┘
```

## Performance Characteristics

```
┌───────────────────────────────────────────────────────┐
│                 Operation Complexity                  │
├───────────────────────────────────────────────────────┤
│                                                       │
│  CRUD Operations:                                    │
│  ┌─────────────────────────┬───────────────────┐    │
│  │ add_snippet()           │ O(1) avg          │    │
│  │ remove_snippet()        │ O(1) avg          │    │
│  │ update_snippet()        │ O(1) avg          │    │
│  │ get_snippet()           │ O(1) avg          │    │
│  └─────────────────────────┴───────────────────┘    │
│                                                       │
│  Search Operations:                                  │
│  ┌─────────────────────────┬───────────────────┐    │
│  │ find_exact_trigger()    │ O(1) avg          │    │
│  │ find_by_trigger()       │ O(k) k=matches    │    │
│  │ get_by_category()       │ O(m) m=in category│    │
│  │ get_all_snippets()      │ O(n) n=total      │    │
│  └─────────────────────────┴───────────────────┘    │
│                                                       │
│  Template Operations:                                │
│  ┌─────────────────────────┬───────────────────┐    │
│  │ parse_template()        │ O(n) n=length     │    │
│  │ expand_template()       │ O(n+p) p=pholders │    │
│  └─────────────────────────┴───────────────────┘    │
│                                                       │
│  Persistence:                                        │
│  ┌─────────────────────────┬───────────────────┐    │
│  │ save_to_file()          │ O(n) n=snippets   │    │
│  │ load_from_file()        │ O(n) n=snippets   │    │
│  └─────────────────────────┴───────────────────┘    │
│                                                       │
└───────────────────────────────────────────────────────┘
```

## Memory Layout

```
SnippetManager (typical with 50 snippets):
┌────────────────────────────────────────┐
│ snippets: ~10KB                        │  Main storage
│   50 snippets × ~200 bytes each        │
├────────────────────────────────────────┤
│ trigger_index: ~2KB                    │  Fast lookup
│   50 entries × ~40 bytes each          │
├────────────────────────────────────────┤
│ category_index: ~1KB                   │  Category grouping
│   5 categories × ~200 bytes each       │
├────────────────────────────────────────┤
│ Total: ~13KB                           │  Lightweight!
└────────────────────────────────────────┘
```

## Thread Safety

```
Current: Not thread-safe (single-threaded usage)

To make thread-safe:
┌────────────────────────────────────────────┐
│  Option 1: Arc<RwLock<SnippetManager>>    │
│  - Multiple readers, single writer         │
│  - Minimal lock contention                 │
└────────────────────────────────────────────┘

┌────────────────────────────────────────────┐
│  Option 2: Arc<Mutex<SnippetManager>>     │
│  - Simpler, exclusive access               │
│  - Good for low contention                 │
└────────────────────────────────────────────┘
```

## Extension Points

```
Future Enhancements:
├─→ Variable System
│   ├─→ $DATE, $TIME, $USER
│   ├─→ $CLIPBOARD
│   └─→ $RANDOM_UUID
│
├─→ Transformations
│   ├─→ ${name|uppercase}
│   ├─→ ${text|snakecase}
│   └─→ ${text|camelcase}
│
├─→ Conditional Blocks
│   ├─→ #{if:var}...#{endif}
│   └─→ Show/hide based on values
│
└─→ Snippet Inheritance
    ├─→ Extend base snippets
    └─→ Override placeholders
```

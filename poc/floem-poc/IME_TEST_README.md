# Floem Korean IME Test Application

## Overview

This is a test application built with Floem 0.2 to verify Korean IME (Input Method Editor) functionality.

## File Location

`/Users/yunwoopc/SIDE-PROJECT/agterm/poc/floem-poc/src/ime_test.rs`

## Features

### 1. Text Input Field
- Text input widget with placeholder text
- Real-time signal-based state management using `RwSignal`
- Placeholder: "여기에 입력하세요..." (Please enter here...)

### 2. Real-time Display
- Shows entered text immediately as you type
- Format: "입력된 텍스트: {text}" (Entered text: {text})

### 3. Character Analysis
- **Character Count**: Shows both character count and byte length
  - Example: "글자 수: 5 (바이트: 15)"
  - Useful for verifying UTF-8 encoding

### 4. Detailed Character Breakdown
- Displays each character individually with Unicode codepoint
- Format: `[index] 'char' (U+XXXX)`
- Example output:
  ```
  문자 분석:
  [0] '안' (U+C548)
  [1] '녕' (U+B155)
  ```

### 5. IME Status
- Shows IME state (currently displays "IME: Ready")
- Green colored text for easy visibility

### 6. Test Examples
Provides test cases for Korean IME:
- Simple Hangul: 안녕하세요
- Complex composition: 닭, 삶, 앉다
- Mixed English: Hello 안녕
- With numbers: 2024년 1월

## How to Run

```bash
# Build
cargo build --bin ime-test

# Run
cargo run --bin ime-test

# Release build (optimized)
cargo build --bin ime-test --release
cargo run --bin ime-test --release
```

## Testing Guide

### Test Cases

1. **Simple Hangul Input**
   - Type: `안녕하세요`
   - Verify: Each character displays correctly
   - Check: Character count matches visual count

2. **Complex Consonant Clusters**
   - Type: `닭` (dalk), `삶` (salm), `앉다` (anjda)
   - Verify: Final consonant clusters render properly
   - Check: Unicode codepoints are correct

3. **IME Composition**
   - Start typing Korean (e.g., type 'ㅎ')
   - Continue composition (add 'ㅏ' → '하')
   - Verify: Character updates during composition

4. **Mixed Input**
   - Type: `Hello 안녕 123`
   - Verify: All character types display correctly
   - Check: Byte count reflects UTF-8 encoding

5. **Editing**
   - Type some text
   - Use backspace/delete
   - Move cursor with arrow keys
   - Verify: All editing operations work correctly

## Architecture

### State Management
```rust
let input_text = RwSignal::new(String::new());
```
- Uses Floem's reactive signal system
- Automatically updates UI when signal changes

### Layout Structure
```
v_stack
├── Title label
├── Instructions label
├── Text input widget
├── Display label (reactive)
├── Character count label (reactive)
├── Character analysis label (reactive)
├── IME status label (reactive)
└── Test examples label
```

### Styling
- Uses `.style()` method for component styling
- Supports font size, padding, margin, colors
- Border radius for modern look
- Light gray background (#F5F5F5)

## Known Behaviors

### Unicode Representation
- Korean characters use UTF-8 encoding
- Each Hangul syllable is typically 3 bytes
- Example: '한' = 0xED 0x95 0x9C = U+D55C

### IME Composition States
1. **Pre-composition**: Individual jamo not yet combined
2. **Composing**: Jamo being combined into syllable
3. **Committed**: Final syllable entered

### Character Breakdown
The app shows Unicode codepoints to help debug:
- Hangul syllables: U+AC00 to U+D7A3
- Hangul jamo (parts): U+1100 to U+11FF
- Can reveal if IME is sending composed vs uncomposed characters

## Troubleshooting

### Build Issues
If you encounter build issues:
```bash
# Clean and rebuild
cargo clean
cargo build --bin ime-test

# Or use release mode (faster, more memory efficient)
cargo build --bin ime-test --release
```

### IME Not Working
- Ensure your system has Korean IME enabled
- macOS: System Preferences > Keyboard > Input Sources
- Linux: Configure ibus or fcitx
- Windows: Settings > Time & Language > Language

### Display Issues
- If Korean text shows as boxes, ensure Korean fonts are installed
- Floem uses system fonts by default
- Check console for font-related warnings

## Expected Behavior

### Correct IME Flow
1. User activates Korean IME
2. Types consonant (e.g., 'ㅎ')
3. Types vowel (e.g., 'ㅏ') → combines to '하'
4. Types final consonant (optional, e.g., 'ㄴ') → becomes '한'
5. Presses space or types new syllable → commits '한'

### Visual Feedback
- Text should appear in input field immediately
- Display labels update in real-time
- Character analysis shows correct Unicode values
- No lag or stuttering during composition

## API Reference

### Key Floem Components Used

```rust
// Reactive state
RwSignal::new(initial_value)
signal.get()  // Read value
signal.set()  // Write value

// Widgets
label(|| "text")           // Static or dynamic label
text_input(signal)          // Text input bound to signal
v_stack((widgets...))       // Vertical stack layout

// Styling
.style(|s| s.property(value))
```

### Available Style Properties
- `font_size(f32)`
- `font_weight(u16)`
- `padding(f32)`
- `margin(f32)`
- `margin_top(f32)`
- `width(f32)` / `width_full()`
- `height(f32)` / `height_full()`
- `border(f32)`
- `border_radius(f32)`
- `color(Color)`
- `background(Color)`
- `line_height(f32)`
- `font_family(String)`

## Future Enhancements

Potential improvements:
1. Display IME composition state (pre-edit text)
2. Show cursor position
3. Add text selection visualization
4. Display input method name
5. Add more complex test patterns
6. Multi-line text input testing
7. Performance metrics for large text
8. Copy/paste testing

## Related Documentation

- [Floem GitHub](https://github.com/lapce/floem)
- [Floem Examples](https://github.com/lapce/floem/tree/main/examples)
- [Unicode Hangul](https://en.wikipedia.org/wiki/Hangul)
- [IME Wikipedia](https://en.wikipedia.org/wiki/Input_method)

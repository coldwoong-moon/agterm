//! Markdown Syntax Highlighting for AgTerm

use iced::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkdownToken {
    Header(u8),
    Bold,
    Italic,
    Code,
    CodeBlock,
    Link,
    ListItem,
    Blockquote,
    HorizontalRule,
    Strikethrough,
}

#[derive(Debug, Clone)]
pub struct MarkdownSpan {
    pub start: usize,
    pub end: usize,
    pub token: MarkdownToken,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TextStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
}

pub struct MarkdownHighlighter {
    enabled: bool,
}

impl MarkdownHighlighter {
    pub fn new() -> Self {
        Self { enabled: true }
    }

    pub fn disabled() -> Self {
        Self { enabled: false }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn parse_line(&self, line: &str) -> Vec<MarkdownSpan> {
        if !self.enabled {
            return Vec::new();
        }

        let mut spans = Vec::new();

        if let Some(span) = self.parse_header(line) {
            spans.push(span);
        }

        if let Some(span) = self.parse_horizontal_rule(line) {
            spans.push(span);
            return spans;
        }

        if let Some(span) = self.parse_blockquote(line) {
            spans.push(span);
        }

        if let Some(span) = self.parse_list_item(line) {
            spans.push(span);
        }

        spans.extend(self.parse_inline_elements(line));

        spans
    }

    pub fn token_color(&self, token: MarkdownToken) -> Color {
        match token {
            MarkdownToken::Header(_) => Color::from_rgb(0.36, 0.54, 0.98),
            MarkdownToken::Bold => Color::from_rgb(0.95, 0.77, 0.36),
            MarkdownToken::Italic => Color::from_rgb(0.55, 0.36, 0.98),
            MarkdownToken::Code => Color::from_rgb(0.35, 0.78, 0.55),
            MarkdownToken::CodeBlock => Color::from_rgb(0.35, 0.78, 0.55),
            MarkdownToken::Link => Color::from_rgb(0.36, 0.54, 0.98),
            MarkdownToken::ListItem => Color::from_rgb(0.95, 0.77, 0.36),
            MarkdownToken::Blockquote => Color::from_rgb(0.6, 0.62, 0.68),
            MarkdownToken::HorizontalRule => Color::from_rgb(0.6, 0.62, 0.68),
            MarkdownToken::Strikethrough => Color::from_rgb(0.92, 0.39, 0.45),
        }
    }

    pub fn token_style(&self, token: MarkdownToken) -> TextStyle {
        match token {
            MarkdownToken::Header(_) => TextStyle {
                bold: true,
                italic: false,
                underline: false,
                strikethrough: false,
            },
            MarkdownToken::Bold => TextStyle {
                bold: true,
                italic: false,
                underline: false,
                strikethrough: false,
            },
            MarkdownToken::Italic => TextStyle {
                bold: false,
                italic: true,
                underline: false,
                strikethrough: false,
            },
            MarkdownToken::Code | MarkdownToken::CodeBlock => TextStyle {
                bold: false,
                italic: false,
                underline: false,
                strikethrough: false,
            },
            MarkdownToken::Link => TextStyle {
                bold: false,
                italic: false,
                underline: true,
                strikethrough: false,
            },
            MarkdownToken::Strikethrough => TextStyle {
                bold: false,
                italic: false,
                underline: false,
                strikethrough: true,
            },
            _ => TextStyle::default(),
        }
    }

    fn parse_header(&self, line: &str) -> Option<MarkdownSpan> {
        let trimmed = line.trim_start();
        if !trimmed.starts_with('#') {
            return None;
        }

        let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
        if hash_count > 6 {
            return None;
        }

        if trimmed.len() > hash_count && !trimmed.chars().nth(hash_count)?.is_whitespace() {
            return None;
        }

        Some(MarkdownSpan {
            start: 0,
            end: line.len(),
            token: MarkdownToken::Header(hash_count as u8),
        })
    }

    fn parse_horizontal_rule(&self, line: &str) -> Option<MarkdownSpan> {
        let trimmed = line.trim();

        if trimmed.len() < 3 {
            return None;
        }

        let chars: Vec<char> = trimmed.chars().collect();
        let first_char = chars[0];

        if first_char != '-' && first_char != '*' && first_char != '_' {
            return None;
        }

        if chars.iter().all(|&c| c == first_char || c.is_whitespace())
            && chars.iter().filter(|&&c| c == first_char).count() >= 3
        {
            return Some(MarkdownSpan {
                start: 0,
                end: line.len(),
                token: MarkdownToken::HorizontalRule,
            });
        }

        None
    }

    fn parse_blockquote(&self, line: &str) -> Option<MarkdownSpan> {
        let trimmed = line.trim_start();
        if trimmed.starts_with("> ") {
            Some(MarkdownSpan {
                start: 0,
                end: 2 + (line.len() - trimmed.len()),
                token: MarkdownToken::Blockquote,
            })
        } else {
            None
        }
    }

    fn parse_list_item(&self, line: &str) -> Option<MarkdownSpan> {
        let trimmed = line.trim_start();
        if let Some(first_char) = trimmed.chars().next() {
            if (first_char == '-' || first_char == '*' || first_char == '+')
                && trimmed.len() > 1
                && trimmed.chars().nth(1)?.is_whitespace()
            {
                return Some(MarkdownSpan {
                    start: line.len() - trimmed.len(),
                    end: line.len() - trimmed.len() + 2,
                    token: MarkdownToken::ListItem,
                });
            }
        }
        None
    }

    fn parse_inline_elements(&self, line: &str) -> Vec<MarkdownSpan> {
        let mut spans = Vec::new();

        spans.extend(self.find_delimited(line, "```", MarkdownToken::CodeBlock));
        spans.extend(self.find_delimited(line, "`", MarkdownToken::Code));
        spans.extend(self.find_delimited(line, "**", MarkdownToken::Bold));
        spans.extend(self.find_delimited(line, "__", MarkdownToken::Bold));
        spans.extend(self.find_delimited(line, "*", MarkdownToken::Italic));
        spans.extend(self.find_delimited(line, "_", MarkdownToken::Italic));
        spans.extend(self.find_delimited(line, "~~", MarkdownToken::Strikethrough));
        spans.extend(self.find_links(line));

        spans
    }

    fn find_delimited(
        &self,
        line: &str,
        delimiter: &str,
        token: MarkdownToken,
    ) -> Vec<MarkdownSpan> {
        let mut spans = Vec::new();
        let mut start_pos = None;
        let mut pos = 0;
        let delimiter_len = delimiter.len();

        while pos <= line.len().saturating_sub(delimiter_len) {
            if &line[pos..pos + delimiter_len] == delimiter {
                if let Some(start) = start_pos {
                    spans.push(MarkdownSpan {
                        start,
                        end: pos + delimiter_len,
                        token,
                    });
                    start_pos = None;
                    pos += delimiter_len;
                } else {
                    start_pos = Some(pos);
                    pos += delimiter_len;
                }
            } else {
                pos += 1;
            }
        }

        spans
    }

    fn find_links(&self, line: &str) -> Vec<MarkdownSpan> {
        let mut spans = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '[' {
                if let Some(text_end) = chars[i..].iter().position(|&c| c == ']') {
                    let text_end = i + text_end;
                    if text_end + 1 < chars.len() && chars[text_end + 1] == '(' {
                        if let Some(url_end) = chars[text_end + 1..].iter().position(|&c| c == ')')
                        {
                            let url_end = text_end + 1 + url_end;
                            spans.push(MarkdownSpan {
                                start: i,
                                end: url_end + 1,
                                token: MarkdownToken::Link,
                            });
                            i = url_end + 1;
                            continue;
                        }
                    }
                }
            }
            i += 1;
        }

        spans
    }
}

impl Default for MarkdownHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_parsing() {
        let highlighter = MarkdownHighlighter::new();

        let spans = highlighter.parse_line("# Header 1");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].token, MarkdownToken::Header(1));

        let spans = highlighter.parse_line("### Header 3");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].token, MarkdownToken::Header(3));

        let spans = highlighter.parse_line("###### Header 6");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].token, MarkdownToken::Header(6));
    }

    #[test]
    fn test_bold_parsing() {
        let highlighter = MarkdownHighlighter::new();

        let spans = highlighter.parse_line("This is **bold** text");
        let bold_spans: Vec<_> = spans
            .iter()
            .filter(|s| s.token == MarkdownToken::Bold)
            .collect();
        assert_eq!(bold_spans.len(), 1);
        assert_eq!(
            &"This is **bold** text"[bold_spans[0].start..bold_spans[0].end],
            "**bold**"
        );
    }

    #[test]
    fn test_italic_parsing() {
        let highlighter = MarkdownHighlighter::new();

        let spans = highlighter.parse_line("This is *italic* text");
        let italic_spans: Vec<_> = spans
            .iter()
            .filter(|s| s.token == MarkdownToken::Italic)
            .collect();
        assert_eq!(italic_spans.len(), 1);
    }

    #[test]
    fn test_code_parsing() {
        let highlighter = MarkdownHighlighter::new();

        let spans = highlighter.parse_line("This is `code` text");
        let code_spans: Vec<_> = spans
            .iter()
            .filter(|s| s.token == MarkdownToken::Code)
            .collect();
        assert_eq!(code_spans.len(), 1);
    }

    #[test]
    fn test_code_block_parsing() {
        let highlighter = MarkdownHighlighter::new();

        let spans = highlighter.parse_line("```rust```");
        let code_block_spans: Vec<_> = spans
            .iter()
            .filter(|s| s.token == MarkdownToken::CodeBlock)
            .collect();
        assert_eq!(code_block_spans.len(), 1);
    }

    #[test]
    fn test_link_parsing() {
        let highlighter = MarkdownHighlighter::new();

        let spans = highlighter.parse_line("Check [this link](https://example.com)");
        let link_spans: Vec<_> = spans
            .iter()
            .filter(|s| s.token == MarkdownToken::Link)
            .collect();
        assert_eq!(link_spans.len(), 1);
        assert_eq!(
            &"Check [this link](https://example.com)"[link_spans[0].start..link_spans[0].end],
            "[this link](https://example.com)"
        );
    }

    #[test]
    fn test_list_item_parsing() {
        let highlighter = MarkdownHighlighter::new();

        let spans = highlighter.parse_line("- List item");
        assert!(spans.iter().any(|s| s.token == MarkdownToken::ListItem));

        let spans = highlighter.parse_line("* List item");
        assert!(spans.iter().any(|s| s.token == MarkdownToken::ListItem));

        let spans = highlighter.parse_line("+ List item");
        assert!(spans.iter().any(|s| s.token == MarkdownToken::ListItem));
    }

    #[test]
    fn test_blockquote_parsing() {
        let highlighter = MarkdownHighlighter::new();

        let spans = highlighter.parse_line("> This is a quote");
        assert!(spans.iter().any(|s| s.token == MarkdownToken::Blockquote));
    }

    #[test]
    fn test_horizontal_rule_parsing() {
        let highlighter = MarkdownHighlighter::new();

        let spans = highlighter.parse_line("---");
        assert!(spans
            .iter()
            .any(|s| s.token == MarkdownToken::HorizontalRule));

        let spans = highlighter.parse_line("***");
        assert!(spans
            .iter()
            .any(|s| s.token == MarkdownToken::HorizontalRule));

        let spans = highlighter.parse_line("___");
        assert!(spans
            .iter()
            .any(|s| s.token == MarkdownToken::HorizontalRule));
    }

    #[test]
    fn test_strikethrough_parsing() {
        let highlighter = MarkdownHighlighter::new();

        let spans = highlighter.parse_line("This is ~~deleted~~ text");
        let strike_spans: Vec<_> = spans
            .iter()
            .filter(|s| s.token == MarkdownToken::Strikethrough)
            .collect();
        assert_eq!(strike_spans.len(), 1);
    }

    #[test]
    fn test_mixed_formatting() {
        let highlighter = MarkdownHighlighter::new();

        let spans = highlighter.parse_line("**bold** and *italic* and `code`");
        assert!(spans.iter().any(|s| s.token == MarkdownToken::Bold));
        assert!(spans.iter().any(|s| s.token == MarkdownToken::Italic));
        assert!(spans.iter().any(|s| s.token == MarkdownToken::Code));
    }

    #[test]
    fn test_token_colors() {
        let highlighter = MarkdownHighlighter::new();

        let _color = highlighter.token_color(MarkdownToken::Header(1));
        let _color = highlighter.token_color(MarkdownToken::Bold);
        let _color = highlighter.token_color(MarkdownToken::Code);
    }

    #[test]
    fn test_token_styles() {
        let highlighter = MarkdownHighlighter::new();

        let style = highlighter.token_style(MarkdownToken::Bold);
        assert!(style.bold);
        assert!(!style.italic);

        let style = highlighter.token_style(MarkdownToken::Italic);
        assert!(!style.bold);
        assert!(style.italic);

        let style = highlighter.token_style(MarkdownToken::Link);
        assert!(style.underline);
    }

    #[test]
    fn test_disabled_highlighter() {
        let highlighter = MarkdownHighlighter::disabled();

        let spans = highlighter.parse_line("# Header with **bold**");
        assert_eq!(spans.len(), 0);
    }

    #[test]
    fn test_enable_disable() {
        let mut highlighter = MarkdownHighlighter::new();
        assert!(highlighter.is_enabled());

        highlighter.set_enabled(false);
        assert!(!highlighter.is_enabled());

        highlighter.set_enabled(true);
        assert!(highlighter.is_enabled());
    }
}

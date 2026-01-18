//! Internationalization (i18n) support for AgTerm
//!
//! This module provides comprehensive internationalization features including:
//! - Locale parsing and management (ISO 639-1, ISO 3166-1, ISO 15924)
//! - Message catalogs with fallback support
//! - Placeholder and template formatting
//! - Pluralization rules for different languages
//! - Number formatting (integers, decimals, currency, percentages)
//! - Date and time formatting (relative and absolute)
//! - RTL (Right-to-Left) language support
//! - JSON-based message catalog loading
//!
//! # Examples
//!
//! ```
//! use agterm::i18n::{I18n, Locale};
//!
//! let mut i18n = I18n::new();
//! i18n.set_locale(Locale::from_str("ko-KR").unwrap());
//! let message = i18n.t("terminal.new_tab");
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// =============================================================================
// Error Types
// =============================================================================

/// Errors that can occur during i18n operations
#[derive(Debug, Clone, thiserror::Error)]
pub enum I18nError {
    #[error("Invalid locale format: {0}")]
    InvalidLocale(String),

    #[error("Catalog not found for locale: {0}")]
    CatalogNotFound(String),

    #[error("Message not found: {0}")]
    MessageNotFound(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Format error: {0}")]
    FormatError(String),
}

/// Alias for Result with I18nError
pub type Result<T> = std::result::Result<T, I18nError>;

// =============================================================================
// Locale
// =============================================================================

/// Represents a locale with language, region, script, and variant
///
/// Follows BCP 47 language tag format (e.g., "en-US", "zh-Hans-CN")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Locale {
    /// ISO 639-1 language code (e.g., "en", "ko", "ja")
    pub language: String,

    /// ISO 3166-1 region code (e.g., "US", "KR", "JP")
    pub region: Option<String>,

    /// ISO 15924 script code (e.g., "Latn", "Hang", "Hant")
    pub script: Option<String>,

    /// Variant (e.g., "1996" for German orthography)
    pub variant: Option<String>,
}

impl Locale {
    /// Creates a new locale with only a language code
    ///
    /// # Examples
    ///
    /// ```
    /// use agterm::i18n::Locale;
    /// let locale = Locale::new("en");
    /// assert_eq!(locale.language, "en");
    /// ```
    pub fn new(language: &str) -> Self {
        Self {
            language: language.to_lowercase(),
            region: None,
            script: None,
            variant: None,
        }
    }

    /// Creates a new locale with language and region
    ///
    /// # Examples
    ///
    /// ```
    /// use agterm::i18n::Locale;
    /// let locale = Locale::with_region("en", "US");
    /// assert_eq!(locale.language, "en");
    /// assert_eq!(locale.region, Some("US".to_string()));
    /// ```
    pub fn with_region(language: &str, region: &str) -> Self {
        Self {
            language: language.to_lowercase(),
            region: Some(region.to_uppercase()),
            script: None,
            variant: None,
        }
    }

    /// Parses a locale from a string (e.g., "en-US", "zh-Hans-CN")
    ///
    /// # Examples
    ///
    /// ```
    /// use agterm::i18n::Locale;
    /// let locale = Locale::from_str("en-US").unwrap();
    /// assert_eq!(locale.language, "en");
    /// assert_eq!(locale.region, Some("US".to_string()));
    /// ```
    pub fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('-').collect();

        if parts.is_empty() {
            return Err(I18nError::InvalidLocale("Empty locale string".to_string()));
        }

        let language = parts[0].to_lowercase();
        if language.len() != 2 && language.len() != 3 {
            return Err(I18nError::InvalidLocale(
                format!("Invalid language code: {}", language)
            ));
        }

        let mut locale = Self {
            language,
            region: None,
            script: None,
            variant: None,
        };

        // Parse remaining parts
        for part in parts.iter().skip(1) {
            match part.len() {
                2 => {
                    // Region code (2 letters)
                    if part.chars().all(|c| c.is_ascii_alphabetic()) {
                        locale.region = Some(part.to_uppercase());
                    } else {
                        return Err(I18nError::InvalidLocale(
                            format!("Invalid region code: {}", part)
                        ));
                    }
                }
                3 => {
                    // Could be 3-letter language extension or variant
                    locale.variant = Some(part.to_lowercase());
                }
                4 => {
                    // Script code (4 letters)
                    if part.chars().all(|c| c.is_ascii_alphabetic()) {
                        let mut chars = part.chars();
                        let first = chars.next().unwrap().to_uppercase().collect::<String>();
                        let rest = chars.as_str().to_lowercase();
                        locale.script = Some(format!("{}{}", first, rest));
                    } else {
                        return Err(I18nError::InvalidLocale(
                            format!("Invalid script code: {}", part)
                        ));
                    }
                }
                _ => {
                    // Variant (5-8 letters or digits)
                    locale.variant = Some(part.to_lowercase());
                }
            }
        }

        Ok(locale)
    }

    /// Converts the locale to a string representation
    ///
    /// # Examples
    ///
    /// ```
    /// use agterm::i18n::Locale;
    /// let locale = Locale::with_region("en", "US");
    /// assert_eq!(locale.to_bcp47(), "en-US");
    /// ```
    pub fn to_bcp47(&self) -> String {
        let mut parts = vec![self.language.clone()];

        if let Some(script) = &self.script {
            parts.push(script.clone());
        }

        if let Some(region) = &self.region {
            parts.push(region.clone());
        }

        if let Some(variant) = &self.variant {
            parts.push(variant.clone());
        }

        parts.join("-")
    }

    /// Checks if this locale matches another locale (fuzzy matching)
    ///
    /// Allows matching with partial information (e.g., "en" matches "en-US")
    ///
    /// # Examples
    ///
    /// ```
    /// use agterm::i18n::Locale;
    /// let locale1 = Locale::new("en");
    /// let locale2 = Locale::with_region("en", "US");
    /// assert!(locale1.matches(&locale2));
    /// ```
    pub fn matches(&self, other: &Locale) -> bool {
        if self.language != other.language {
            return false;
        }

        if let (Some(s1), Some(s2)) = (&self.script, &other.script) {
            if s1 != s2 {
                return false;
            }
        }

        if let (Some(r1), Some(r2)) = (&self.region, &other.region) {
            if r1 != r2 {
                return false;
            }
        }

        true
    }

    /// Gets the system's default locale
    pub fn system_default() -> Self {
        // Try to get locale from environment variables
        std::env::var("LANG")
            .or_else(|_| std::env::var("LC_ALL"))
            .or_else(|_| std::env::var("LC_MESSAGES"))
            .ok()
            .and_then(|lang| {
                // Parse format like "en_US.UTF-8"
                let locale_part = lang.split('.').next()?;
                let normalized = locale_part.replace('_', "-");
                Locale::from_str(&normalized).ok()
            })
            .unwrap_or_else(|| Locale::new("en"))
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_bcp47())
    }
}

// =============================================================================
// Message Keys
// =============================================================================

/// Type-safe message keys for compile-time safety
pub mod keys {
    // Terminal operations
    pub const TERMINAL_NEW_TAB: &str = "terminal.new_tab";
    pub const TERMINAL_CLOSE_TAB: &str = "terminal.close_tab";
    pub const TERMINAL_COPY: &str = "terminal.copy";
    pub const TERMINAL_PASTE: &str = "terminal.paste";
    pub const TERMINAL_CLEAR: &str = "terminal.clear";
    pub const TERMINAL_SEARCH: &str = "terminal.search";
    pub const TERMINAL_FIND_NEXT: &str = "terminal.find_next";
    pub const TERMINAL_FIND_PREV: &str = "terminal.find_prev";
    pub const TERMINAL_SPLIT_HORIZONTAL: &str = "terminal.split_horizontal";
    pub const TERMINAL_SPLIT_VERTICAL: &str = "terminal.split_vertical";

    // Settings
    pub const SETTINGS_TITLE: &str = "settings.title";
    pub const SETTINGS_GENERAL: &str = "settings.general";
    pub const SETTINGS_APPEARANCE: &str = "settings.appearance";
    pub const SETTINGS_KEYBOARD: &str = "settings.keyboard";
    pub const SETTINGS_PROFILES: &str = "settings.profiles";
    pub const SETTINGS_ADVANCED: &str = "settings.advanced";

    // Common
    pub const COMMON_OK: &str = "common.ok";
    pub const COMMON_CANCEL: &str = "common.cancel";
    pub const COMMON_SAVE: &str = "common.save";
    pub const COMMON_DELETE: &str = "common.delete";
    pub const COMMON_EDIT: &str = "common.edit";
    pub const COMMON_CONFIRM: &str = "common.confirm";
    pub const COMMON_CLOSE: &str = "common.close";
    pub const COMMON_APPLY: &str = "common.apply";

    // Errors
    pub const ERROR_GENERIC: &str = "error.generic";
    pub const ERROR_CONNECTION_FAILED: &str = "error.connection_failed";
    pub const ERROR_TIMEOUT: &str = "error.timeout";
    pub const ERROR_PERMISSION_DENIED: &str = "error.permission_denied";
    pub const ERROR_FILE_NOT_FOUND: &str = "error.file_not_found";
}

// =============================================================================
// Message Catalog
// =============================================================================

/// A collection of localized messages for a specific locale
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageCatalog {
    /// The locale this catalog is for
    pub locale: Locale,

    /// Key-value pairs of message IDs to localized strings
    pub messages: HashMap<String, String>,
}

impl MessageCatalog {
    /// Creates a new empty message catalog
    pub fn new(locale: Locale) -> Self {
        Self {
            locale,
            messages: HashMap::new(),
        }
    }

    /// Gets a message by key
    pub fn get(&self, key: &str) -> Option<&str> {
        self.messages.get(key).map(|s| s.as_str())
    }

    /// Gets a message by key or returns a default value
    pub fn get_with_default<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.get(key).unwrap_or(default)
    }

    /// Formats a message with placeholder replacement
    ///
    /// Placeholders are in the format `{name}` and are replaced with values from args
    ///
    /// # Examples
    ///
    /// ```
    /// use agterm::i18n::{MessageCatalog, Locale};
    /// let mut catalog = MessageCatalog::new(Locale::new("en"));
    /// catalog.insert("greeting", "Hello, {name}!");
    /// let result = catalog.format("greeting", &[("name", "Alice")]);
    /// assert_eq!(result, "Hello, Alice!");
    /// ```
    pub fn format(&self, key: &str, args: &[(&str, &str)]) -> String {
        let template = self.get(key).unwrap_or(key);
        let mut result = template.to_string();

        for (name, value) in args {
            let placeholder = format!("{{{}}}", name);
            result = result.replace(&placeholder, value);
        }

        result
    }

    /// Inserts a message into the catalog
    pub fn insert(&mut self, key: &str, value: &str) {
        self.messages.insert(key.to_string(), value.to_string());
    }

    /// Merges another catalog into this one
    ///
    /// Messages from the other catalog will override existing messages
    pub fn merge(&mut self, other: MessageCatalog) {
        self.messages.extend(other.messages);
    }

    /// Loads messages from a JSON string
    pub fn from_json(locale: Locale, json: &str) -> Result<Self> {
        let messages: HashMap<String, String> = serde_json::from_str(json)
            .map_err(|e| I18nError::ParseError(e.to_string()))?;

        Ok(Self { locale, messages })
    }
}

// =============================================================================
// Pluralization
// =============================================================================

/// Plural categories as defined by CLDR (Unicode Common Locale Data Repository)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluralCategory {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
}

/// Determines the plural form for a given locale and count
///
/// This is a simplified implementation. A full implementation would use CLDR rules.
pub fn plural_form(locale: &Locale, count: usize) -> PluralCategory {
    match locale.language.as_str() {
        "en" => {
            // English: one (1), other
            if count == 1 {
                PluralCategory::One
            } else {
                PluralCategory::Other
            }
        }
        "ko" | "ja" | "zh" | "th" | "vi" => {
            // East Asian languages: other (no plural distinction)
            PluralCategory::Other
        }
        "ru" | "uk" => {
            // Slavic languages: one (1, 21, 31...), few (2-4, 22-24...), many (0, 5-20, 25-30...)
            let n = count % 100;
            let n10 = count % 10;

            if n10 == 1 && n != 11 {
                PluralCategory::One
            } else if (2..=4).contains(&n10) && !(12..=14).contains(&n) {
                PluralCategory::Few
            } else {
                PluralCategory::Many
            }
        }
        "ar" => {
            // Arabic: zero (0), one (1), two (2), few (3-10), many (11-99), other
            match count {
                0 => PluralCategory::Zero,
                1 => PluralCategory::One,
                2 => PluralCategory::Two,
                3..=10 => PluralCategory::Few,
                11..=99 => PluralCategory::Many,
                _ => PluralCategory::Other,
            }
        }
        "pl" => {
            // Polish: one (1), few (2-4, 22-24...), many (0, 5-21, 25-31...)
            let n = count % 100;
            let n10 = count % 10;

            if count == 1 {
                PluralCategory::One
            } else if (2..=4).contains(&n10) && !(12..=14).contains(&n) {
                PluralCategory::Few
            } else {
                PluralCategory::Many
            }
        }
        _ => {
            // Default: one (1), other
            if count == 1 {
                PluralCategory::One
            } else {
                PluralCategory::Other
            }
        }
    }
}

// =============================================================================
// Number Formatting
// =============================================================================

/// Formats a number according to locale conventions
pub fn format_number(value: f64, locale: &Locale) -> String {
    match locale.language.as_str() {
        "en" => {
            // English: 1,234.56
            format_with_separators(value, ',', '.')
        }
        "ko" | "ja" | "zh" => {
            // East Asian: 1,234.56 (same as English in modern usage)
            format_with_separators(value, ',', '.')
        }
        "de" | "es" | "it" | "pt" | "ru" => {
            // European: 1.234,56
            format_with_separators(value, '.', ',')
        }
        "fr" => {
            // French: 1 234,56
            format_with_separators(value, ' ', ',')
        }
        _ => {
            // Default
            format!("{}", value)
        }
    }
}

/// Formats an integer according to locale conventions
pub fn format_integer(value: i64, locale: &Locale) -> String {
    let float_val = value as f64;
    let formatted = format_number(float_val, locale);

    // Determine decimal separator based on locale
    let decimal_sep = match locale.language.as_str() {
        "en" | "ko" | "ja" | "zh" => '.',
        "de" | "es" | "it" | "pt" | "ru" | "fr" => ',',
        _ => '.',
    };

    // Remove decimal part if present
    if let Some(pos) = formatted.find(decimal_sep) {
        formatted[..pos].to_string()
    } else {
        formatted
    }
}

/// Formats currency according to locale conventions
pub fn format_currency(value: f64, currency: &str, locale: &Locale) -> String {
    let number_part = format_number(value, locale);

    match locale.language.as_str() {
        "en" => {
            match currency {
                "USD" => format!("${}", number_part),
                "EUR" => format!("€{}", number_part),
                "GBP" => format!("£{}", number_part),
                _ => format!("{} {}", number_part, currency),
            }
        }
        "ko" => {
            match currency {
                "KRW" => format!("₩{}", number_part),
                "USD" => format!("${}", number_part),
                _ => format!("{}{}", number_part, currency),
            }
        }
        "ja" => {
            match currency {
                "JPY" => format!("¥{}", number_part),
                "USD" => format!("${}", number_part),
                _ => format!("{}{}", number_part, currency),
            }
        }
        "de" | "fr" | "es" | "it" => {
            match currency {
                "EUR" => format!("{} €", number_part),
                "USD" => format!("{} $", number_part),
                _ => format!("{} {}", number_part, currency),
            }
        }
        _ => {
            format!("{} {}", number_part, currency)
        }
    }
}

/// Formats a percentage according to locale conventions
pub fn format_percent(value: f64, locale: &Locale) -> String {
    let number_part = format_number(value * 100.0, locale);
    format!("{}%", number_part)
}

fn format_with_separators(value: f64, thousands_sep: char, decimal_sep: char) -> String {
    let negative = value < 0.0;
    let abs_value = value.abs();

    let integer_part = abs_value.trunc() as i64;
    let decimal_part = abs_value.fract();

    // Format integer part with thousands separator
    let mut int_str = integer_part.to_string();
    let mut chars: Vec<char> = int_str.chars().collect();
    let mut i = chars.len();

    while i > 3 {
        i -= 3;
        chars.insert(i, thousands_sep);
    }

    int_str = chars.into_iter().collect();

    // Format decimal part
    let result = if decimal_part > 0.0 {
        let dec_str = format!("{:.2}", decimal_part).trim_start_matches("0.").to_string();
        format!("{}{}{}", int_str, decimal_sep, dec_str)
    } else {
        int_str
    };

    if negative {
        format!("-{}", result)
    } else {
        result
    }
}

// =============================================================================
// Date/Time Formatting
// =============================================================================

/// Date format styles
#[derive(Debug, Clone, Copy)]
pub enum DateFormat {
    Short,   // 1/1/24
    Medium,  // Jan 1, 2024
    Long,    // January 1, 2024
    Full,    // Monday, January 1, 2024
}

/// Time format styles
#[derive(Debug, Clone, Copy)]
pub enum TimeFormat {
    Short,   // 1:30 PM
    Medium,  // 1:30:45 PM
    Long,    // 1:30:45 PM UTC
}

/// Combined date/time format
#[derive(Debug, Clone, Copy)]
pub enum DateTimeFormat {
    Short,
    Medium,
    Long,
}

/// Formats a date according to locale and format style
pub fn format_date(dt: &DateTime<Utc>, format: DateFormat, locale: &Locale) -> String {
    let format_str = match (locale.language.as_str(), format) {
        ("en", DateFormat::Short) => "%m/%d/%y",
        ("en", DateFormat::Medium) => "%b %d, %Y",
        ("en", DateFormat::Long) => "%B %d, %Y",
        ("en", DateFormat::Full) => "%A, %B %d, %Y",

        ("ko", DateFormat::Short) => "%y. %m. %d.",
        ("ko", DateFormat::Medium) => "%Y년 %m월 %d일",
        ("ko", DateFormat::Long) => "%Y년 %m월 %d일",
        ("ko", DateFormat::Full) => "%Y년 %m월 %d일 %A",

        ("ja", DateFormat::Short) => "%y/%m/%d",
        ("ja", DateFormat::Medium) => "%Y年%m月%d日",
        ("ja", DateFormat::Long) => "%Y年%m月%d日",
        ("ja", DateFormat::Full) => "%Y年%m月%d日 %A",

        ("de", DateFormat::Short) => "%d.%m.%y",
        ("de", DateFormat::Medium) => "%d. %b %Y",
        ("de", DateFormat::Long) => "%d. %B %Y",
        ("de", DateFormat::Full) => "%A, %d. %B %Y",

        _ => "%Y-%m-%d",
    };

    dt.format(format_str).to_string()
}

/// Formats a time according to locale and format style
pub fn format_time(dt: &DateTime<Utc>, format: TimeFormat, locale: &Locale) -> String {
    let format_str = match (locale.language.as_str(), format) {
        ("en", TimeFormat::Short) => "%I:%M %p",
        ("en", TimeFormat::Medium) => "%I:%M:%S %p",
        ("en", TimeFormat::Long) => "%I:%M:%S %p %Z",

        ("ko" | "ja" | "de", TimeFormat::Short) => "%H:%M",
        ("ko" | "ja" | "de", TimeFormat::Medium) => "%H:%M:%S",
        ("ko" | "ja" | "de", TimeFormat::Long) => "%H:%M:%S %Z",

        _ => "%H:%M:%S",
    };

    dt.format(format_str).to_string()
}

/// Formats a date and time according to locale and format style
pub fn format_datetime(dt: &DateTime<Utc>, format: DateTimeFormat, locale: &Locale) -> String {
    let date_fmt = match format {
        DateTimeFormat::Short => DateFormat::Short,
        DateTimeFormat::Medium => DateFormat::Medium,
        DateTimeFormat::Long => DateFormat::Long,
    };

    let time_fmt = match format {
        DateTimeFormat::Short => TimeFormat::Short,
        DateTimeFormat::Medium => TimeFormat::Medium,
        DateTimeFormat::Long => TimeFormat::Long,
    };

    let date_str = format_date(dt, date_fmt, locale);
    let time_str = format_time(dt, time_fmt, locale);

    format!("{} {}", date_str, time_str)
}

/// Formats a relative time (e.g., "2 hours ago", "in 3 days")
pub fn format_relative(dt: &DateTime<Utc>, locale: &Locale) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(*dt);

    let (value, unit, is_past) = if duration.num_seconds().abs() < 60 {
        (duration.num_seconds().abs(), "second", duration.num_seconds() >= 0)
    } else if duration.num_minutes().abs() < 60 {
        (duration.num_minutes().abs(), "minute", duration.num_minutes() >= 0)
    } else if duration.num_hours().abs() < 24 {
        (duration.num_hours().abs(), "hour", duration.num_hours() >= 0)
    } else if duration.num_days().abs() < 30 {
        (duration.num_days().abs(), "day", duration.num_days() >= 0)
    } else if duration.num_days().abs() < 365 {
        (duration.num_days().abs() / 30, "month", duration.num_days() >= 0)
    } else {
        (duration.num_days().abs() / 365, "year", duration.num_days() >= 0)
    };

    match locale.language.as_str() {
        "en" => {
            let plural = if value != 1 { "s" } else { "" };
            if is_past {
                format!("{} {}{} ago", value, unit, plural)
            } else {
                format!("in {} {}{}", value, unit, plural)
            }
        }
        "ko" => {
            let unit_kr = match unit {
                "second" => "초",
                "minute" => "분",
                "hour" => "시간",
                "day" => "일",
                "month" => "개월",
                "year" => "년",
                _ => unit,
            };
            if is_past {
                format!("{}{} 전", value, unit_kr)
            } else {
                format!("{}{} 후", value, unit_kr)
            }
        }
        "ja" => {
            let unit_ja = match unit {
                "second" => "秒",
                "minute" => "分",
                "hour" => "時間",
                "day" => "日",
                "month" => "ヶ月",
                "year" => "年",
                _ => unit,
            };
            if is_past {
                format!("{}{}前", value, unit_ja)
            } else {
                format!("{}{}後", value, unit_ja)
            }
        }
        _ => {
            if is_past {
                format!("{} {} ago", value, unit)
            } else {
                format!("in {} {}", value, unit)
            }
        }
    }
}

// =============================================================================
// RTL Support
// =============================================================================

/// Text direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDirection {
    LeftToRight,
    RightToLeft,
}

/// Determines if a locale uses right-to-left text direction
pub fn is_rtl(locale: &Locale) -> bool {
    matches!(
        locale.language.as_str(),
        "ar" | "he" | "fa" | "ur" | "yi"
    )
}

/// Gets the text direction for a locale
pub fn text_direction(locale: &Locale) -> TextDirection {
    if is_rtl(locale) {
        TextDirection::RightToLeft
    } else {
        TextDirection::LeftToRight
    }
}

// =============================================================================
// I18n Manager
// =============================================================================

/// Main internationalization manager
pub struct I18n {
    /// Message catalogs for different locales
    catalogs: HashMap<Locale, MessageCatalog>,

    /// Current active locale
    current_locale: Locale,

    /// Fallback locale (used when message not found in current locale)
    fallback_locale: Locale,
}

impl I18n {
    /// Creates a new I18n manager with system locale
    pub fn new() -> Self {
        let system_locale = Locale::system_default();
        let fallback = Locale::new("en");

        let mut i18n = Self {
            catalogs: HashMap::new(),
            current_locale: system_locale,
            fallback_locale: fallback.clone(),
        };

        // Load built-in English messages
        i18n.load_catalog(fallback.clone(), Self::english_messages());

        // Load built-in Korean messages
        i18n.load_catalog(Locale::new("ko"), Self::korean_messages());

        // Load built-in Japanese messages
        i18n.load_catalog(Locale::new("ja"), Self::japanese_messages());

        i18n
    }

    /// Creates a new I18n manager with a specific locale
    pub fn with_locale(locale: Locale) -> Self {
        let mut i18n = Self::new();
        i18n.current_locale = locale;
        i18n
    }

    /// Sets the current locale
    pub fn set_locale(&mut self, locale: Locale) {
        self.current_locale = locale;
    }

    /// Gets the current locale
    pub fn current_locale(&self) -> &Locale {
        &self.current_locale
    }

    /// Gets a list of available locales
    pub fn available_locales(&self) -> Vec<&Locale> {
        self.catalogs.keys().collect()
    }

    /// Translates a message key to the current locale
    ///
    /// Falls back to the fallback locale if the message is not found
    pub fn t(&self, key: &str) -> String {
        // Try current locale
        if let Some(catalog) = self.find_catalog(&self.current_locale) {
            if let Some(msg) = catalog.get(key) {
                return msg.to_string();
            }
        }

        // Try fallback locale
        if let Some(catalog) = self.find_catalog(&self.fallback_locale) {
            if let Some(msg) = catalog.get(key) {
                return msg.to_string();
            }
        }

        // Return the key itself as last resort
        key.to_string()
    }

    /// Translates a message with placeholder arguments
    pub fn t_with_args(&self, key: &str, args: &[(&str, &str)]) -> String {
        let template = self.t(key);
        let mut result = template;

        for (name, value) in args {
            let placeholder = format!("{{{}}}", name);
            result = result.replace(&placeholder, value);
        }

        result
    }

    /// Translates a message with plural support
    pub fn t_plural(&self, key: &str, count: usize) -> String {
        let category = plural_form(&self.current_locale, count);
        let plural_key = format!("{}_{:?}", key, category).to_lowercase();

        // Try plural-specific key first
        let msg = self.t(&plural_key);
        if msg != plural_key {
            return msg.replace("{count}", &count.to_string());
        }

        // Fall back to regular key
        self.t(key).replace("{count}", &count.to_string())
    }

    /// Loads a message catalog for a locale
    pub fn load_catalog(&mut self, locale: Locale, catalog: MessageCatalog) {
        self.catalogs.insert(locale, catalog);
    }

    /// Loads a message catalog from a JSON string
    pub fn load_from_json(&mut self, locale: Locale, json: &str) -> Result<()> {
        let catalog = MessageCatalog::from_json(locale.clone(), json)?;
        self.load_catalog(locale, catalog);
        Ok(())
    }

    /// Finds a catalog for a locale with fuzzy matching
    fn find_catalog(&self, locale: &Locale) -> Option<&MessageCatalog> {
        // Exact match
        if let Some(catalog) = self.catalogs.get(locale) {
            return Some(catalog);
        }

        // Fuzzy match
        for (cat_locale, catalog) in &self.catalogs {
            if locale.matches(cat_locale) {
                return Some(catalog);
            }
        }

        None
    }

    /// Built-in English messages
    fn english_messages() -> MessageCatalog {
        let mut catalog = MessageCatalog::new(Locale::new("en"));

        // Terminal operations
        catalog.insert(keys::TERMINAL_NEW_TAB, "New Tab");
        catalog.insert(keys::TERMINAL_CLOSE_TAB, "Close Tab");
        catalog.insert(keys::TERMINAL_COPY, "Copy");
        catalog.insert(keys::TERMINAL_PASTE, "Paste");
        catalog.insert(keys::TERMINAL_CLEAR, "Clear");
        catalog.insert(keys::TERMINAL_SEARCH, "Search");
        catalog.insert(keys::TERMINAL_FIND_NEXT, "Find Next");
        catalog.insert(keys::TERMINAL_FIND_PREV, "Find Previous");
        catalog.insert(keys::TERMINAL_SPLIT_HORIZONTAL, "Split Horizontally");
        catalog.insert(keys::TERMINAL_SPLIT_VERTICAL, "Split Vertically");

        // Settings
        catalog.insert(keys::SETTINGS_TITLE, "Settings");
        catalog.insert(keys::SETTINGS_GENERAL, "General");
        catalog.insert(keys::SETTINGS_APPEARANCE, "Appearance");
        catalog.insert(keys::SETTINGS_KEYBOARD, "Keyboard");
        catalog.insert(keys::SETTINGS_PROFILES, "Profiles");
        catalog.insert(keys::SETTINGS_ADVANCED, "Advanced");

        // Common
        catalog.insert(keys::COMMON_OK, "OK");
        catalog.insert(keys::COMMON_CANCEL, "Cancel");
        catalog.insert(keys::COMMON_SAVE, "Save");
        catalog.insert(keys::COMMON_DELETE, "Delete");
        catalog.insert(keys::COMMON_EDIT, "Edit");
        catalog.insert(keys::COMMON_CONFIRM, "Confirm");
        catalog.insert(keys::COMMON_CLOSE, "Close");
        catalog.insert(keys::COMMON_APPLY, "Apply");

        // Errors
        catalog.insert(keys::ERROR_GENERIC, "An error occurred");
        catalog.insert(keys::ERROR_CONNECTION_FAILED, "Connection failed");
        catalog.insert(keys::ERROR_TIMEOUT, "Operation timed out");
        catalog.insert(keys::ERROR_PERMISSION_DENIED, "Permission denied");
        catalog.insert(keys::ERROR_FILE_NOT_FOUND, "File not found");

        catalog
    }

    /// Built-in Korean messages
    fn korean_messages() -> MessageCatalog {
        let mut catalog = MessageCatalog::new(Locale::new("ko"));

        // Terminal operations
        catalog.insert(keys::TERMINAL_NEW_TAB, "새 탭");
        catalog.insert(keys::TERMINAL_CLOSE_TAB, "탭 닫기");
        catalog.insert(keys::TERMINAL_COPY, "복사");
        catalog.insert(keys::TERMINAL_PASTE, "붙여넣기");
        catalog.insert(keys::TERMINAL_CLEAR, "지우기");
        catalog.insert(keys::TERMINAL_SEARCH, "검색");
        catalog.insert(keys::TERMINAL_FIND_NEXT, "다음 찾기");
        catalog.insert(keys::TERMINAL_FIND_PREV, "이전 찾기");
        catalog.insert(keys::TERMINAL_SPLIT_HORIZONTAL, "가로 분할");
        catalog.insert(keys::TERMINAL_SPLIT_VERTICAL, "세로 분할");

        // Settings
        catalog.insert(keys::SETTINGS_TITLE, "설정");
        catalog.insert(keys::SETTINGS_GENERAL, "일반");
        catalog.insert(keys::SETTINGS_APPEARANCE, "모양");
        catalog.insert(keys::SETTINGS_KEYBOARD, "키보드");
        catalog.insert(keys::SETTINGS_PROFILES, "프로필");
        catalog.insert(keys::SETTINGS_ADVANCED, "고급");

        // Common
        catalog.insert(keys::COMMON_OK, "확인");
        catalog.insert(keys::COMMON_CANCEL, "취소");
        catalog.insert(keys::COMMON_SAVE, "저장");
        catalog.insert(keys::COMMON_DELETE, "삭제");
        catalog.insert(keys::COMMON_EDIT, "편집");
        catalog.insert(keys::COMMON_CONFIRM, "확인");
        catalog.insert(keys::COMMON_CLOSE, "닫기");
        catalog.insert(keys::COMMON_APPLY, "적용");

        // Errors
        catalog.insert(keys::ERROR_GENERIC, "오류가 발생했습니다");
        catalog.insert(keys::ERROR_CONNECTION_FAILED, "연결 실패");
        catalog.insert(keys::ERROR_TIMEOUT, "시간 초과");
        catalog.insert(keys::ERROR_PERMISSION_DENIED, "권한 거부됨");
        catalog.insert(keys::ERROR_FILE_NOT_FOUND, "파일을 찾을 수 없음");

        catalog
    }

    /// Built-in Japanese messages
    fn japanese_messages() -> MessageCatalog {
        let mut catalog = MessageCatalog::new(Locale::new("ja"));

        // Terminal operations
        catalog.insert(keys::TERMINAL_NEW_TAB, "新しいタブ");
        catalog.insert(keys::TERMINAL_CLOSE_TAB, "タブを閉じる");
        catalog.insert(keys::TERMINAL_COPY, "コピー");
        catalog.insert(keys::TERMINAL_PASTE, "貼り付け");
        catalog.insert(keys::TERMINAL_CLEAR, "クリア");
        catalog.insert(keys::TERMINAL_SEARCH, "検索");
        catalog.insert(keys::TERMINAL_FIND_NEXT, "次を検索");
        catalog.insert(keys::TERMINAL_FIND_PREV, "前を検索");
        catalog.insert(keys::TERMINAL_SPLIT_HORIZONTAL, "水平分割");
        catalog.insert(keys::TERMINAL_SPLIT_VERTICAL, "垂直分割");

        // Settings
        catalog.insert(keys::SETTINGS_TITLE, "設定");
        catalog.insert(keys::SETTINGS_GENERAL, "一般");
        catalog.insert(keys::SETTINGS_APPEARANCE, "外観");
        catalog.insert(keys::SETTINGS_KEYBOARD, "キーボード");
        catalog.insert(keys::SETTINGS_PROFILES, "プロファイル");
        catalog.insert(keys::SETTINGS_ADVANCED, "詳細");

        // Common
        catalog.insert(keys::COMMON_OK, "OK");
        catalog.insert(keys::COMMON_CANCEL, "キャンセル");
        catalog.insert(keys::COMMON_SAVE, "保存");
        catalog.insert(keys::COMMON_DELETE, "削除");
        catalog.insert(keys::COMMON_EDIT, "編集");
        catalog.insert(keys::COMMON_CONFIRM, "確認");
        catalog.insert(keys::COMMON_CLOSE, "閉じる");
        catalog.insert(keys::COMMON_APPLY, "適用");

        // Errors
        catalog.insert(keys::ERROR_GENERIC, "エラーが発生しました");
        catalog.insert(keys::ERROR_CONNECTION_FAILED, "接続失敗");
        catalog.insert(keys::ERROR_TIMEOUT, "タイムアウト");
        catalog.insert(keys::ERROR_PERMISSION_DENIED, "権限が拒否されました");
        catalog.insert(keys::ERROR_FILE_NOT_FOUND, "ファイルが見つかりません");

        catalog
    }
}

impl Default for I18n {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_new() {
        let locale = Locale::new("en");
        assert_eq!(locale.language, "en");
        assert_eq!(locale.region, None);
        assert_eq!(locale.script, None);
        assert_eq!(locale.variant, None);
    }

    #[test]
    fn test_locale_with_region() {
        let locale = Locale::with_region("en", "US");
        assert_eq!(locale.language, "en");
        assert_eq!(locale.region, Some("US".to_string()));
    }

    #[test]
    fn test_locale_from_str_simple() {
        let locale = Locale::from_str("en").unwrap();
        assert_eq!(locale.language, "en");
        assert_eq!(locale.region, None);
    }

    #[test]
    fn test_locale_from_str_with_region() {
        let locale = Locale::from_str("en-US").unwrap();
        assert_eq!(locale.language, "en");
        assert_eq!(locale.region, Some("US".to_string()));
    }

    #[test]
    fn test_locale_from_str_with_script() {
        let locale = Locale::from_str("zh-Hans-CN").unwrap();
        assert_eq!(locale.language, "zh");
        assert_eq!(locale.script, Some("Hans".to_string()));
        assert_eq!(locale.region, Some("CN".to_string()));
    }

    #[test]
    fn test_locale_to_string() {
        let locale = Locale::with_region("en", "US");
        assert_eq!(locale.to_string(), "en-US");
    }

    #[test]
    fn test_locale_matches() {
        let locale1 = Locale::new("en");
        let locale2 = Locale::with_region("en", "US");
        assert!(locale1.matches(&locale2));
        assert!(locale2.matches(&locale1));
    }

    #[test]
    fn test_locale_no_match() {
        let locale1 = Locale::new("en");
        let locale2 = Locale::new("ko");
        assert!(!locale1.matches(&locale2));
    }

    #[test]
    fn test_message_catalog_get() {
        let mut catalog = MessageCatalog::new(Locale::new("en"));
        catalog.insert("test.key", "Test Value");
        assert_eq!(catalog.get("test.key"), Some("Test Value"));
    }

    #[test]
    fn test_message_catalog_get_missing() {
        let catalog = MessageCatalog::new(Locale::new("en"));
        assert_eq!(catalog.get("missing.key"), None);
    }

    #[test]
    fn test_message_catalog_format() {
        let mut catalog = MessageCatalog::new(Locale::new("en"));
        catalog.insert("greeting", "Hello, {name}!");
        let result = catalog.format("greeting", &[("name", "Alice")]);
        assert_eq!(result, "Hello, Alice!");
    }

    #[test]
    fn test_message_catalog_format_multiple() {
        let mut catalog = MessageCatalog::new(Locale::new("en"));
        catalog.insert("message", "{greeting}, {name}! You have {count} messages.");
        let result = catalog.format(
            "message",
            &[("greeting", "Hi"), ("name", "Bob"), ("count", "5")],
        );
        assert_eq!(result, "Hi, Bob! You have 5 messages.");
    }

    #[test]
    fn test_plural_form_english() {
        let locale = Locale::new("en");
        assert_eq!(plural_form(&locale, 0), PluralCategory::Other);
        assert_eq!(plural_form(&locale, 1), PluralCategory::One);
        assert_eq!(plural_form(&locale, 2), PluralCategory::Other);
    }

    #[test]
    fn test_plural_form_korean() {
        let locale = Locale::new("ko");
        assert_eq!(plural_form(&locale, 0), PluralCategory::Other);
        assert_eq!(plural_form(&locale, 1), PluralCategory::Other);
        assert_eq!(plural_form(&locale, 100), PluralCategory::Other);
    }

    #[test]
    fn test_format_number_english() {
        let locale = Locale::new("en");
        let result = format_number(1234.56, &locale);
        assert!(result.contains("1") && result.contains("234"));
    }

    #[test]
    fn test_format_integer() {
        let locale = Locale::new("en");
        let result = format_integer(1234567, &locale);
        assert!(result.contains("1") && result.contains("234"));
        assert!(!result.contains('.'));
    }

    #[test]
    fn test_format_currency_usd() {
        let locale = Locale::new("en");
        let result = format_currency(1234.56, "USD", &locale);
        assert!(result.contains('$'));
    }

    #[test]
    fn test_format_percent() {
        let locale = Locale::new("en");
        let result = format_percent(0.75, &locale);
        assert!(result.contains("75"));
        assert!(result.contains('%'));
    }

    #[test]
    fn test_is_rtl() {
        assert!(is_rtl(&Locale::new("ar")));
        assert!(is_rtl(&Locale::new("he")));
        assert!(!is_rtl(&Locale::new("en")));
        assert!(!is_rtl(&Locale::new("ko")));
    }

    #[test]
    fn test_text_direction() {
        assert_eq!(text_direction(&Locale::new("ar")), TextDirection::RightToLeft);
        assert_eq!(text_direction(&Locale::new("en")), TextDirection::LeftToRight);
    }

    #[test]
    fn test_i18n_new() {
        let i18n = I18n::new();
        assert_eq!(i18n.fallback_locale.language, "en");
    }

    #[test]
    fn test_i18n_translate_english() {
        let i18n = I18n::new();
        let msg = i18n.t(keys::TERMINAL_NEW_TAB);
        assert_eq!(msg, "New Tab");
    }

    #[test]
    fn test_i18n_translate_korean() {
        let mut i18n = I18n::new();
        i18n.set_locale(Locale::new("ko"));
        let msg = i18n.t(keys::TERMINAL_NEW_TAB);
        assert_eq!(msg, "새 탭");
    }

    #[test]
    fn test_i18n_translate_japanese() {
        let mut i18n = I18n::new();
        i18n.set_locale(Locale::new("ja"));
        let msg = i18n.t(keys::TERMINAL_COPY);
        assert_eq!(msg, "コピー");
    }

    #[test]
    fn test_i18n_fallback() {
        let mut i18n = I18n::new();
        i18n.set_locale(Locale::new("fr")); // French not available
        let msg = i18n.t(keys::TERMINAL_NEW_TAB);
        assert_eq!(msg, "New Tab"); // Falls back to English
    }

    #[test]
    fn test_i18n_missing_key() {
        let i18n = I18n::new();
        let msg = i18n.t("missing.key");
        assert_eq!(msg, "missing.key"); // Returns key itself
    }

    #[test]
    fn test_i18n_translate_with_args() {
        let mut i18n = I18n::new();
        let mut catalog = MessageCatalog::new(Locale::new("en"));
        catalog.insert("welcome", "Welcome, {name}!");
        i18n.load_catalog(Locale::new("en"), catalog);

        let msg = i18n.t_with_args("welcome", &[("name", "Alice")]);
        assert_eq!(msg, "Welcome, Alice!");
    }

    #[test]
    fn test_message_catalog_from_json() {
        let json = r#"{"test.key": "Test Value", "another.key": "Another Value"}"#;
        let catalog = MessageCatalog::from_json(Locale::new("en"), json).unwrap();
        assert_eq!(catalog.get("test.key"), Some("Test Value"));
        assert_eq!(catalog.get("another.key"), Some("Another Value"));
    }

    #[test]
    fn test_i18n_load_from_json() {
        let mut i18n = I18n::new();
        let json = r#"{"custom.key": "Custom Value"}"#;
        i18n.load_from_json(Locale::new("en"), json).unwrap();

        let msg = i18n.t("custom.key");
        assert_eq!(msg, "Custom Value");
    }

    #[test]
    fn test_format_date_english() {
        let dt = Utc::now();
        let locale = Locale::new("en");
        let result = format_date(&dt, DateFormat::Medium, &locale);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_format_time_english() {
        let dt = Utc::now();
        let locale = Locale::new("en");
        let result = format_time(&dt, TimeFormat::Short, &locale);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_format_relative() {
        let locale = Locale::new("en");
        let now = Utc::now();

        // Test past time
        let past = now - chrono::Duration::hours(2);
        let result = format_relative(&past, &locale);
        assert!(result.contains("ago"));

        // Test future time
        let future = now + chrono::Duration::hours(3);
        let result = format_relative(&future, &locale);
        assert!(result.contains("in"));
    }

    #[test]
    fn test_available_locales() {
        let i18n = I18n::new();
        let locales = i18n.available_locales();
        assert!(locales.len() >= 3); // At least en, ko, ja
    }
}

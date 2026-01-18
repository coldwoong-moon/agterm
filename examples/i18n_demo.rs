//! Demonstration of the i18n (internationalization) module
//!
//! Run with: `cargo run --example i18n_demo`

use agterm::i18n::{I18n, Locale, keys, format_number, format_currency, format_relative, is_rtl};
use chrono::Utc;

fn main() {
    println!("=== AgTerm i18n Demo ===\n");

    // Create i18n manager
    let mut i18n = I18n::new();

    // English (default)
    println!("--- English ---");
    i18n.set_locale(Locale::new("en"));
    print_messages(&i18n);

    // Korean
    println!("\n--- Korean (한국어) ---");
    i18n.set_locale(Locale::new("ko"));
    print_messages(&i18n);

    // Japanese
    println!("\n--- Japanese (日本語) ---");
    i18n.set_locale(Locale::new("ja"));
    print_messages(&i18n);

    // Number formatting
    println!("\n=== Number Formatting ===");
    demonstrate_number_formatting();

    // Date formatting
    println!("\n=== Date/Time Formatting ===");
    demonstrate_date_formatting();

    // RTL support
    println!("\n=== RTL Detection ===");
    demonstrate_rtl_detection();

    // Custom messages with placeholders
    println!("\n=== Custom Messages ===");
    demonstrate_custom_messages(&i18n);
}

fn print_messages(i18n: &I18n) {
    println!("New Tab: {}", i18n.t(keys::TERMINAL_NEW_TAB));
    println!("Close Tab: {}", i18n.t(keys::TERMINAL_CLOSE_TAB));
    println!("Copy: {}", i18n.t(keys::TERMINAL_COPY));
    println!("Paste: {}", i18n.t(keys::TERMINAL_PASTE));
    println!("Settings: {}", i18n.t(keys::SETTINGS_TITLE));
    println!("OK: {}", i18n.t(keys::COMMON_OK));
    println!("Cancel: {}", i18n.t(keys::COMMON_CANCEL));
    println!("Error: {}", i18n.t(keys::ERROR_GENERIC));
}

fn demonstrate_number_formatting() {
    let number = 1234567.89;

    let locales = vec![
        ("English", Locale::new("en")),
        ("Korean", Locale::new("ko")),
        ("Japanese", Locale::new("ja")),
        ("German", Locale::new("de")),
        ("French", Locale::new("fr")),
    ];

    for (name, locale) in locales {
        println!("{}: {}", name, format_number(number, &locale));
        println!("  Currency: {}", format_currency(number, "USD", &locale));
    }
}

fn demonstrate_date_formatting() {
    let now = Utc::now();
    let past = now - chrono::Duration::hours(2);

    let locales = vec![
        ("English", Locale::new("en")),
        ("Korean", Locale::new("ko")),
        ("Japanese", Locale::new("ja")),
    ];

    for (name, locale) in locales {
        println!("{}: {}", name, format_relative(&past, &locale));
    }
}

fn demonstrate_rtl_detection() {
    let locales = vec![
        ("English", Locale::new("en"), false),
        ("Korean", Locale::new("ko"), false),
        ("Arabic", Locale::new("ar"), true),
        ("Hebrew", Locale::new("he"), true),
    ];

    for (name, locale, expected_rtl) in locales {
        let is_rtl_lang = is_rtl(&locale);
        println!("{}: RTL={} (expected: {})", name, is_rtl_lang, expected_rtl);
    }
}

fn demonstrate_custom_messages(i18n: &I18n) {
    let greeting = i18n.t_with_args(
        "greeting",
        &[("name", "Alice"), ("app", "AgTerm")]
    );
    println!("Custom greeting: {}", greeting);
}

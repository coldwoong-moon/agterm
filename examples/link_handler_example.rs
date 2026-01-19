//! Link Handler Usage Example
//!
//! This example demonstrates how to use the link_handler module to detect
//! and handle various types of links in terminal output.

use agterm::link_handler::{LinkAction, LinkDetector, LinkHandler, LinkType};

fn main() {
    // Example 1: Basic link detection
    println!("Example 1: Basic Link Detection");
    println!("=================================");

    let detector = LinkDetector::new();
    let text = "Visit https://example.com or email support@example.com";
    let links = detector.detect_links(text);

    println!("Text: {}", text);
    println!("Found {} links:", links.len());
    for link in &links {
        println!("  - Type: {:?}, Text: '{}', Position: {}..{}",
                 link.link_type, link.text, link.start, link.end);
    }
    println!();

    // Example 2: Find link at position
    println!("Example 2: Find Link at Position");
    println!("==================================");

    let text2 = "Check /var/log/app.log or connect to 192.168.1.1:8080";
    println!("Text: {}", text2);

    if let Some(link) = detector.find_link_at(text2, 10) {
        println!("Link at position 10: '{}' ({})", link.text, link.link_type.name());
    }

    if let Some(link) = detector.find_link_at(text2, 45) {
        println!("Link at position 45: '{}' ({})", link.text, link.link_type.name());
    }
    println!();

    // Example 3: Custom link patterns
    println!("Example 3: Custom Link Patterns");
    println!("================================");

    let mut custom_detector = LinkDetector::new();

    // Add pattern for GitHub issue references (#123)
    custom_detector
        .add_custom_pattern("GitHub Issue".to_string(), r"#\d+")
        .unwrap();

    // Add pattern for Jira tickets (PROJ-123)
    custom_detector
        .add_custom_pattern("Jira Ticket".to_string(), r"[A-Z]+-\d+")
        .unwrap();

    let text3 = "Fixed in #123 and also addresses PROJ-456";
    let links3 = custom_detector.detect_links(text3);

    println!("Text: {}", text3);
    println!("Found {} links:", links3.len());
    for link in &links3 {
        println!("  - Type: {:?}, Text: '{}'", link.link_type, link.text);
    }
    println!();

    // Example 4: Link handler with actions
    println!("Example 4: Link Handler with Actions");
    println!("=====================================");

    let mut handler = LinkHandler::new();

    // Override default action for email links
    handler.set_default_action(LinkType::Email, LinkAction::CopyToClipboard);

    // Set custom command for file paths
    handler.set_default_action(
        LinkType::FilePath,
        LinkAction::Command("code {}".to_string())
    );

    let text4 = "Error in /src/main.rs at line 42";
    if let Some(link) = handler.detector().find_link_at(text4, 12) {
        println!("Found link: '{}' ({})", link.text, link.link_type.name());
        println!("Would execute action for this link type");

        // In a real application, you would call:
        // match handler.handle_link(&link) {
        //     Ok(()) => println!("Link handled successfully"),
        //     Err(e) => eprintln!("Error handling link: {}", e),
        // }
    }
    println!();

    // Example 5: Detecting specific link types
    println!("Example 5: Detecting Specific Link Types");
    println!("=========================================");

    // Create detector that only detects URLs and emails
    let filtered_detector = LinkDetector::with_types(&[LinkType::Url, LinkType::Email]);

    let text5 = "Visit https://example.com, email us at info@example.com, or check /var/log";
    let links5 = filtered_detector.detect_links(text5);

    println!("Text: {}", text5);
    println!("Found {} links (URLs and emails only):", links5.len());
    for link in &links5 {
        println!("  - Type: {:?}, Text: '{}'", link.link_type, link.text);
    }
    println!();

    // Example 6: Various URL formats
    println!("Example 6: Various URL Formats");
    println!("===============================");

    let urls = vec![
        "https://example.com/path?query=value&other=123",
        "http://localhost:8080",
        "file:///home/user/document.txt",
        "ftp://ftp.example.com/file.zip",
    ];

    for url in urls {
        if let Some(link) = detector.find_link_at(url, 5) {
            println!("Detected: '{}' as {}", link.text, link.link_type.name());
        }
    }
    println!();

    // Example 7: File path detection
    println!("Example 7: File Path Detection");
    println!("===============================");

    let paths = vec![
        "Error in /usr/local/bin/app.sh:42",
        "Config at ~/config.toml",
        "See ./src/main.rs for details",
        "../README.md has the docs",
    ];

    for path in paths {
        let path_links = detector.detect_links(path);
        for link in path_links {
            if matches!(link.link_type, LinkType::FilePath) {
                println!("Found file path: '{}'", link.text);
            }
        }
    }
    println!();

    // Example 8: IP address detection
    println!("Example 8: IP Address Detection");
    println!("================================");

    let ips = vec![
        "Connect to 192.168.1.1 for admin",
        "Server at 192.168.1.1:8080",
        "Local server [::1]:8080",
        "IPv6 address [2001:db8::1]:443",
    ];

    for ip in ips {
        let ip_links = detector.detect_links(ip);
        for link in ip_links {
            if matches!(link.link_type, LinkType::IpAddress) {
                println!("Found IP: '{}'", link.text);
            }
        }
    }
}

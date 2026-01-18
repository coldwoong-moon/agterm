//! File Diff Utility
//!
//! A simple command-line utility to compare two files using AgTerm's diff viewer.
//!
//! Usage:
//!   cargo run --example diff_files <old_file> <new_file> [--unified] [--width <width>]
//!
//! Examples:
//!   cargo run --example diff_files file1.txt file2.txt
//!   cargo run --example diff_files old.rs new.rs --unified
//!   cargo run --example diff_files old.txt new.txt --width 120

use agterm::diff_view::{DiffViewMode, DiffViewer, MyersDiff};
use std::env;
use std::fs;
use std::process;

fn print_usage() {
    eprintln!("Usage: diff_files <old_file> <new_file> [--unified] [--width <width>]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --unified        Use unified diff view (default: side-by-side)");
    eprintln!("  --width <width>  Set terminal width (default: 100)");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  diff_files file1.txt file2.txt");
    eprintln!("  diff_files old.rs new.rs --unified");
    eprintln!("  diff_files old.txt new.txt --width 120");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        print_usage();
        process::exit(1);
    }

    let old_file = &args[1];
    let new_file = &args[2];

    // Parse options
    let mut mode = DiffViewMode::SideBySide;
    let mut width = 100;

    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--unified" => {
                mode = DiffViewMode::Unified;
                i += 1;
            }
            "--width" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --width requires a value");
                    process::exit(1);
                }
                width = args[i + 1].parse().unwrap_or_else(|_| {
                    eprintln!("Error: Invalid width value");
                    process::exit(1);
                });
                i += 2;
            }
            "--help" | "-h" => {
                print_usage();
                process::exit(0);
            }
            _ => {
                eprintln!("Error: Unknown option: {}", args[i]);
                print_usage();
                process::exit(1);
            }
        }
    }

    // Read files
    let old_content = fs::read_to_string(old_file).unwrap_or_else(|e| {
        eprintln!("Error reading {}: {}", old_file, e);
        process::exit(1);
    });

    let new_content = fs::read_to_string(new_file).unwrap_or_else(|e| {
        eprintln!("Error reading {}: {}", new_file, e);
        process::exit(1);
    });

    // Compute diff
    let old_lines: Vec<String> = old_content.lines().map(|s| s.to_string()).collect();
    let new_lines: Vec<String> = new_content.lines().map(|s| s.to_string()).collect();

    let diff = MyersDiff::new(old_lines, new_lines);
    let result = diff.compute();

    // Display header
    println!("Comparing: {} vs {}", old_file, new_file);
    println!();

    // Display diff
    let mut viewer = DiffViewer::new(result, width);
    viewer.set_mode(mode);
    print!("{}", viewer.render());

    // Display summary
    let stats = viewer.result().stats;
    if stats.total_changes() == 0 {
        println!("\n\x1b[32mFiles are identical\x1b[0m");
    } else {
        println!("\n\x1b[1mSummary:\x1b[0m");
        println!("  Files differ in {} location(s)", stats.total_changes());
        if stats.added > 0 {
            println!("  \x1b[32m+{} line(s) added\x1b[0m", stats.added);
        }
        if stats.removed > 0 {
            println!("  \x1b[31m-{} line(s) removed\x1b[0m", stats.removed);
        }
        if stats.modified > 0 {
            println!("  \x1b[33m~{} line(s) modified\x1b[0m", stats.modified);
        }
    }
}

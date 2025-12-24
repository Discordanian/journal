use chrono::{Local, Timelike};
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    // Verify environment variables
    let journal_home = env::var("JOURNAL_HOME")
        .map_err(|_| "JOURNAL_HOME environment variable is not set".to_string())?;

    let journal_format = env::var("JOURNAL_FORMAT")
        .map_err(|_| "JOURNAL_FORMAT environment variable is not set".to_string())?;

    // Verify JOURNAL_HOME exists and is a directory
    let home_path = PathBuf::from(&journal_home);
    
    // Canonicalize the path to resolve symlinks and normalize the path
    let home_path = home_path
        .canonicalize()
        .map_err(|e| format!("JOURNAL_HOME path cannot be canonicalized: {} ({})", journal_home, e))?;
    
    if !home_path.is_dir() {
        return Err(format!("JOURNAL_HOME is not a directory: {}", journal_home));
    }

    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        return Err("No journal entry provided. Usage: journal <your entry text>".to_string());
    }

    let entry_text = args.join(" ");

    let now = Local::now();
    let hour = now.hour();
    let minute = now.minute();

    let clock_emoji = get_clock_emoji(hour);

    let date_str = format_date(&now, &journal_format)?;

    // Sanitize the date string to prevent path traversal
    let date_str = sanitize_filename(&date_str)?;

    let journal_file = home_path.join(format!("{}.md", date_str));
    
    if !journal_file.starts_with(&home_path) {
        return Err(format!(
            "Invalid journal file path (path traversal detected): {}",
            journal_file.display()
        ));
    }

    let entry_line = format!(
        "📓 {:02}:{:02} {} -> {}\n",
        hour, minute, clock_emoji, entry_text
    );

    // Open the file (will fail if it doesn't exist, avoiding TOCTOU race condition)
    // We check existence by attempting to open, rather than separate exists() check
    let mut file = OpenOptions::new()
        .append(true)
        .open(&journal_file)
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                format!(
                    "Journal file does not exist: {}",
                    journal_file.display()
                )
            } else {
                format!("Failed to open journal file {:?}: {}", journal_file, e)
            }
        })?;

    file.write_all(entry_line.as_bytes())
        .map_err(|e| format!("Failed to write to journal file: {}", e))?;

    println!("✓ Entry added to {}", journal_file.display());

    Ok(())
}

fn get_clock_emoji(hour: u32) -> &'static str {
    match hour % 12 {
        0 => "🕛", // 12 o'clock
        1 => "🕐",
        2 => "🕑",
        3 => "🕒",
        4 => "🕓",
        5 => "🕔",
        6 => "🕕",
        7 => "🕖",
        8 => "🕗",
        9 => "🕘",
        10 => "🕙",
        11 => "🕚",
        _ => "🕛", // Fallback
    }
}

fn format_date(now: &chrono::DateTime<Local>, format: &str) -> Result<String, String> {
    // Convert the JOURNAL_FORMAT to chrono format
    // Common patterns: YYYY-MM-DD, YYYY/MM/DD, DD-MM-YYYY, etc.
    // Replace in order from longest to shortest to avoid partial replacements
    // (e.g., "YYYY" before "YY" to prevent "YYYYMMYYYY" issues)
    let chrono_format = format
        .replace("YYYY", "%Y")
        .replace("YY", "%y")
        .replace("MM", "%m")
        .replace("DD", "%d");

    Ok(now.format(&chrono_format).to_string())
}

/// Sanitizes a string to be used as a filename by removing or replacing dangerous characters
fn sanitize_filename(filename: &str) -> Result<String, String> {
    // Remove path separators and other dangerous characters
    let sanitized: String = filename
        .chars()
        .filter(|c| {
            // Allow alphanumeric, dash, underscore, and period
            c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.'
        })
        .collect();
    
    if sanitized.is_empty() {
        return Err("Date format resulted in empty filename after sanitization".to_string());
    }
    
    // Prevent hidden files (starting with dot) and special names
    if sanitized.starts_with('.') {
        return Err("Date format resulted in hidden filename (starts with dot)".to_string());
    }
    
    // Prevent reserved Windows names (though we're on Unix, it's good practice)
    let upper = sanitized.to_uppercase();
    if matches!(
        upper.as_str(),
        "CON" | "PRN" | "AUX" | "NUL"
            | "COM1" | "COM2" | "COM3" | "COM4" | "COM5" | "COM6" | "COM7" | "COM8" | "COM9"
            | "LPT1" | "LPT2" | "LPT3" | "LPT4" | "LPT5" | "LPT6" | "LPT7" | "LPT8" | "LPT9"
    ) {
        return Err(format!(
            "Date format resulted in reserved filename: {}",
            sanitized
        ));
    }
    
    Ok(sanitized)
}

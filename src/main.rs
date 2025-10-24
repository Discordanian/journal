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
    if !home_path.exists() {
        return Err(format!(
            "JOURNAL_HOME path does not exist: {}",
            journal_home
        ));
    }
    if !home_path.is_dir() {
        return Err(format!("JOURNAL_HOME is not a directory: {}", journal_home));
    }

    // Get command line arguments (skip the program name)
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        return Err("No journal entry provided. Usage: journal <your entry text>".to_string());
    }

    let entry_text = args.join(" ");

    // Get current time
    let now = Local::now();
    let hour = now.hour();
    let minute = now.minute();

    // Get clock emoji based on hour
    let clock_emoji = get_clock_emoji(hour);

    // Format the current date according to JOURNAL_FORMAT
    let date_str = format_date(&now, &journal_format)?;

    // Construct the journal file path
    let journal_file = home_path.join(format!("{}.md", date_str));

    // Create the journal entry line
    let entry_line = format!(
        "JOURNAL CLI {:02}:{:02} {} -> {}\n",
        hour, minute, clock_emoji, entry_text
    );

    // Check if journal file exists
    if !journal_file.exists() {
        return Err(format!(
            "Journal file does not exist: {}",
            journal_file.display()
        ));
    }

    // Append to the journal file
    let mut file = OpenOptions::new()
        .append(true)
        .open(&journal_file)
        .map_err(|e| format!("Failed to open journal file {:?}: {}", journal_file, e))?;

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
    let chrono_format = format
        .replace("YYYY", "%Y")
        .replace("MM", "%m")
        .replace("DD", "%d")
        .replace("YY", "%y");

    Ok(now.format(&chrono_format).to_string())
}

use chrono::{DateTime, Local, NaiveDate};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct Note {
    id: String,
    text: String,
    remind_at: Option<String>,
    done: bool,
    created_at: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("NAVINOTE_URL").unwrap_or_else(|_| "https://notes.navicore.tech".to_string());
    let token = std::env::var("NAVINOTE_TOKEN").expect("NAVINOTE_TOKEN must be set");
    let zet_dir = PathBuf::from(std::env::var("NAVINOTE_ZET_DIR").expect("NAVINOTE_ZET_DIR must be set"));

    let client = reqwest::Client::new();

    // Fetch unsynced notes
    let notes: Vec<Note> = client
        .get(format!("{url}/api/notes?synced=false"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    if notes.is_empty() {
        println!("No unsynced notes.");
        return Ok(());
    }

    // Group by local date
    let mut by_date: BTreeMap<String, Vec<&Note>> = BTreeMap::new();
    for note in &notes {
        let date = if let Ok(dt) = DateTime::parse_from_rfc3339(&note.created_at) {
            dt.with_timezone(&Local).format("%Y-%m-%d").to_string()
        } else {
            note.created_at.get(..10).unwrap_or("unknown").to_string()
        };
        by_date.entry(date).or_default().push(note);
    }

    std::fs::create_dir_all(&zet_dir)?;

    for (date, day_notes) in &by_date {
        let file_path = zet_dir.join(format!("{date}.md"));
        let is_new = !file_path.exists();

        let mut content = if is_new {
            let parsed = NaiveDate::parse_from_str(date, "%Y-%m-%d")
                .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
            format!("---\ntitle: {}\n---\n\n\n", parsed.format("%A, %B %-e, %Y").to_string().replace("  ", " "))
        } else {
            std::fs::read_to_string(&file_path)?
        };

        for note in day_notes {
            let line = if let Some(ra) = &note.remind_at {
                // Parse and reformat to clean ISO8601 without milliseconds
                let formatted_ra = if let Ok(dt) = DateTime::parse_from_rfc3339(ra) {
                    dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()
                } else {
                    ra.clone()
                };
                let checkbox = if note.done { "[x]" } else { "[ ]" };
                format!("* {} #reminder {}: {}\n", checkbox, formatted_ra, note.text)
            } else {
                format!("* {}\n", note.text)
            };
            content.push_str(&line);
        }

        std::fs::write(&file_path, &content)?;
        println!("Wrote {} note(s) to {}", day_notes.len(), file_path.display());
    }

    // Mark all as synced
    for note in &notes {
        client
            .patch(format!("{url}/api/notes/{}/synced", note.id))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await?
            .error_for_status()?;
    }

    println!("Synced {} note(s).", notes.len());
    Ok(())
}

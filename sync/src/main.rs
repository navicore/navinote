use chrono::NaiveDate;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct Note {
    id: String,
    text: String,
    remind_at: Option<String>,
    created_at: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("NAVINOTE_URL").expect("NAVINOTE_URL must be set");
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

    // Group by date
    let mut by_date: BTreeMap<String, Vec<&Note>> = BTreeMap::new();
    for note in &notes {
        let date = note.created_at.get(..10).unwrap_or("unknown");
        by_date.entry(date.to_string()).or_default().push(note);
    }

    std::fs::create_dir_all(&zet_dir)?;

    for (date, day_notes) in &by_date {
        let file_path = zet_dir.join(format!("{date}.md"));
        let is_new = !file_path.exists();

        let mut content = if is_new {
            let parsed = NaiveDate::parse_from_str(date, "%Y-%m-%d")
                .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
            format!("---\ndate: {}\ntags: [quicknote]\n---\n\n# {}\n\n", date, parsed.format("%A, %B %-d, %Y"))
        } else {
            std::fs::read_to_string(&file_path)?
        };

        for note in day_notes {
            let line = if note.remind_at.is_some() {
                let ra = note.remind_at.as_deref().unwrap();
                format!("* [ ] #reminder {}: {} (via quicknote {})\n", ra, note.text, note.created_at)
            } else {
                format!("* {} (via quicknote {})\n", note.text, note.created_at)
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

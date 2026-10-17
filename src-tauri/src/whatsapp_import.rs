//! Bulk-import a WhatsApp "Export chat" text file into the vault's
//! `WhatsApp/<date>.md` daily log. Mirrors the format written by the live
//! clipboard listener (`## HH:MM · Contact\n\n> text\n\n---\n`).
//!
//! Handles the three common WhatsApp export formats:
//!   - `[10/18/26, 14:23] Name: text`           (default, en-US)
//!   - `10/18/2026, 2:23 PM - Name: text`       (locale with AM/PM)
//!   - `10/18/2026, 14:23 - Name: text`         (locale 24h)

use std::{
    collections::BTreeMap,
    fs,
    path::Path,
};

use regex::Regex;
use time::{format_description::FormatItem, macros::format_description, Date, OffsetDateTime, PrimitiveDateTime, Time};

#[derive(Debug, Default, Clone)]
pub struct ImportOutcome {
    pub messages: usize,
    pub days: usize,
    pub skipped: usize,
}

const DAY_FMT: &[FormatItem<'_>] = format_description!("[year]-[month]-[day]");
const VAULT_TS_FMT: &[FormatItem<'_>] = format_description!("[hour]:[minute]");

/// Parse and append a WhatsApp export. `vault` is the resolved vault path.
pub fn import_file(path: &Path, vault: &Path) -> std::io::Result<ImportOutcome> {
    let raw = fs::read_to_string(path)?;
    let parsed = parse(&raw);
    if parsed.is_empty() {
        return Ok(ImportOutcome::default());
    }
    let mut by_day: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for msg in &parsed {
        let day = msg.day.clone();
        by_day
            .entry(day)
            .or_default()
            .push((msg.time.clone(), msg.body.clone()));
    }
    let dir = vault.join("WhatsApp");
    fs::create_dir_all(&dir)?;
    let mut total = 0usize;
    let mut days = 0usize;
    for (day, entries) in by_day {
        let file = dir.join(format!("{day}.md"));
        let mut body = fs::read_to_string(&file).unwrap_or_default();
        for (time, text) in entries {
            if body.contains(&format!("## {time}")) {
                continue;
            }
            body.push_str(&format!("## {time} · imported\n\n> {text}\n\n---\n\n"));
            total += 1;
        }
        if !body.is_empty() {
            fs::write(&file, body)?;
            days += 1;
        }
    }
    Ok(ImportOutcome {
        messages: total,
        days,
        skipped: parsed.iter().filter(|m| m.body.is_empty()).count(),
    })
}

#[derive(Debug, Clone)]
struct ParsedMessage {
    day: String,
    time: String,
    body: String,
}

/// Try every known format. Lines that match none are skipped silently.
fn parse(raw: &str) -> Vec<ParsedMessage> {
    let mut out: Vec<ParsedMessage> = Vec::new();
    let mut current: Option<ParsedMessage> = None;
    let bracketed = Regex::new(r"^\[(\d{1,2}/\d{1,2}/\d{2,4}),\s+(\d{1,2}:\d{2}(?::\d{2})?(?:\s*[AP]M)?)\]\s+([^:]+?):\s*(.*)$").unwrap();
    let dash12 = Regex::new(r"^(\d{1,2}/\d{1,2}/\d{2,4}),\s+(\d{1,2}:\d{2}(?::\d{2})?\s*[AP]M)\s+-\s+([^:]+?):\s*(.*)$").unwrap();
    let dash24 = Regex::new(r"^(\d{1,2}/\d{1,2}/\d{2,4}),\s+(\d{1,2}:\d{2}(?::\d{2})?)\s+-\s+([^:]+?):\s*(.*)$").unwrap();
    for line in raw.lines() {
        let trimmed = line.trim_end_matches('\r');
        if trimmed.is_empty() {
            continue;
        }
        let m = bracketed
            .captures(trimmed)
            .or_else(|| dash12.captures(trimmed))
            .or_else(|| dash24.captures(trimmed));
        if let Some(c) = m {
            if let Some(prev) = current.take() {
                out.push(prev);
            }
            let date_raw = c.get(1).unwrap().as_str();
            let time_raw = c.get(2).unwrap().as_str();
            let _contact = c.get(3).unwrap().as_str();
            let body = c.get(4).unwrap().as_str().to_string();
            let (day, time) = match normalize(date_raw, time_raw) {
                Some(v) => v,
                None => continue,
            };
            current = Some(ParsedMessage { day, time, body });
        } else if let Some(cur) = current.as_mut() {
            // Continuation line of a multi-line message.
            if !cur.body.is_empty() {
                cur.body.push('\n');
            }
            cur.body.push_str(trimmed);
        }
    }
    if let Some(prev) = current {
        out.push(prev);
    }
    out
}

fn normalize(date_raw: &str, time_raw: &str) -> Option<(String, String)> {
    let date = parse_date(date_raw)?;
    let time = parse_time(time_raw)?;
    let now = OffsetDateTime::now_utc();
    let dt = PrimitiveDateTime::new(date, time).assume_utc();
    let local = dt.to_offset(now.offset());
    let day = local.format(&DAY_FMT).ok()?;
    let t = Time::from_hms(local.hour(), local.minute(), 0).ok()?;
    let time = t.format(&VAULT_TS_FMT).ok()?;
    Some((day, time))
}

fn parse_date(s: &str) -> Option<Date> {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() != 3 {
        return None;
    }
    let m: u8 = parts[0].parse().ok()?;
    let d: u8 = parts[1].parse().ok()?;
    let mut y: i32 = parts[2].parse().ok()?;
    if y < 100 {
        y += 2000;
    }
    Date::from_calendar_date(y, time::Month::try_from(m).ok()?, d).ok()
}

fn parse_time(s: &str) -> Option<Time> {
    let s = s.trim();
    let (hms, ampm) = if let Some(rest) = s.strip_suffix("AM").or_else(|| s.strip_suffix("PM")) {
        (rest.trim(), s.ends_with("PM"))
    } else {
        (s, false)
    };
    let mut parts = hms.split(':');
    let h_raw: u8 = parts.next()?.parse().ok()?;
    let m_raw: u8 = parts.next()?.parse().ok()?;
    let sec_raw: u8 = parts
        .next()
        .map(|p| p.parse().unwrap_or(0))
        .unwrap_or(0);
    let mut h = h_raw % 12;
    if ampm && h_raw < 12 {
        h += 12;
    }
    if !ampm && h_raw == 12 {
        h = 0;
    }
    Time::from_hms(h, m_raw, sec_raw).ok()
}

//! The old app stored a kid's grade and guardian inside the Outlook contact's
//! "notes" box, one labelled line each:
//!
//! ```text
//! Grade: 7th
//! Guardian's Name: Jane Doe
//! Guardian's Phone: 555-1234
//! Guardian Email: jane@example.com
//! Last updated: Tuesday, November 10, 2026
//! ```
//!
//! We keep that exact format so existing contacts work unchanged. Any other
//! lines staff typed into the notes box are kept as `extra`.

use chrono::NaiveDate;

const GRADE: &str = "Grade:";
const GUARDIAN_NAME: &str = "Guardian's Name:";
const GUARDIAN_PHONE: &str = "Guardian's Phone:";
const GUARDIAN_EMAIL: &str = "Guardian Email:";
const LAST_UPDATED: &str = "Last updated:";

#[derive(Debug, Default, Clone, PartialEq)]
pub struct NoteFields {
    pub grade: String,
    pub guardian_name: String,
    pub guardian_phone: String,
    pub guardian_email: String,
    pub last_updated: Option<NaiveDate>,
    /// Lines that are not one of ours.
    pub extra: String,
}

pub fn parse(notes: &str) -> NoteFields {
    let mut fields = NoteFields::default();
    let mut extra: Vec<&str> = Vec::new();

    for line in notes.lines() {
        // Phones like to turn ' into a curly quote.
        let plain = line.replace('\u{2019}', "'");

        if let Some(value) = after_label(&plain, GRADE) {
            fields.grade = value;
        } else if let Some(value) = after_label(&plain, GUARDIAN_NAME) {
            fields.guardian_name = value;
        } else if let Some(value) = after_label(&plain, GUARDIAN_PHONE) {
            fields.guardian_phone = value;
        } else if let Some(value) = after_label(&plain, GUARDIAN_EMAIL) {
            fields.guardian_email = value;
        } else if let Some(value) = after_label(&plain, LAST_UPDATED) {
            fields.last_updated = parse_date(&value);
        } else {
            extra.push(line);
        }
    }

    fields.extra = extra.join("\n").trim().to_string();
    fields
}

pub fn format(fields: &NoteFields) -> String {
    let last_updated = match fields.last_updated {
        // Same as C#'s ToLongDateString, which the old app wrote.
        Some(date) => date.format("%A, %B %-d, %Y").to_string(),
        None => String::new(),
    };

    let mut lines = vec![
        format!("{GRADE} {}", fields.grade),
        format!("{GUARDIAN_NAME} {}", fields.guardian_name),
        format!("{GUARDIAN_PHONE} {}", fields.guardian_phone),
        format!("{GUARDIAN_EMAIL} {}", fields.guardian_email),
        format!("{LAST_UPDATED} {last_updated}"),
    ];
    if !fields.extra.is_empty() {
        lines.push(fields.extra.replace('\n', "\r\n"));
    }
    lines.join("\r\n")
}

/// If `line` contains `label`, returns the text after it.
fn after_label(line: &str, label: &str) -> Option<String> {
    let start = line.find(label)? + label.len();
    Some(line[start..].trim().to_string())
}

fn parse_date(text: &str) -> Option<NaiveDate> {
    let formats = ["%A, %B %d, %Y", "%B %d, %Y", "%Y-%m-%d", "%m/%d/%Y"];
    formats
        .iter()
        .find_map(|format| NaiveDate::parse_from_str(text.trim(), format).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> Option<NaiveDate> {
        NaiveDate::from_ymd_opt(y, m, d)
    }

    #[test]
    fn reads_notes_written_by_the_old_app() {
        let notes = "Grade: 7th\r\nGuardian's Name: Jane Doe\r\nGuardian's Phone: 555-1234\r\nGuardian Email: jane@example.com\r\nLast updated: Friday, March 5, 2021";
        let fields = parse(notes);
        assert_eq!(fields.grade, "7th");
        assert_eq!(fields.guardian_name, "Jane Doe");
        assert_eq!(fields.guardian_phone, "555-1234");
        assert_eq!(fields.guardian_email, "jane@example.com");
        assert_eq!(fields.last_updated, date(2021, 3, 5));
        assert_eq!(fields.extra, "");
    }

    #[test]
    fn empty_notes_give_empty_fields() {
        assert_eq!(parse(""), NoteFields::default());
    }

    #[test]
    fn missing_or_empty_values_are_empty() {
        let fields = parse("Grade:\nGuardian's Name: Bob\nLast updated: ");
        assert_eq!(fields.grade, "");
        assert_eq!(fields.guardian_name, "Bob");
        assert_eq!(fields.guardian_phone, "");
        assert_eq!(fields.last_updated, None);
    }

    #[test]
    fn keeps_lines_staff_typed() {
        let fields = parse("Allergic to peanuts\nGrade: 3rd\nPicked up by grandma on Fridays");
        assert_eq!(fields.grade, "3rd");
        assert_eq!(fields.extra, "Allergic to peanuts\nPicked up by grandma on Fridays");
    }

    #[test]
    fn accepts_curly_apostrophes_and_other_date_styles() {
        let fields = parse("Guardian\u{2019}s Phone: 555-0000\nLast updated: 2024-09-01");
        assert_eq!(fields.guardian_phone, "555-0000");
        assert_eq!(fields.last_updated, date(2024, 9, 1));
        assert_eq!(parse("Last updated: 9/1/2024").last_updated, date(2024, 9, 1));
    }

    #[test]
    fn writes_the_old_format_and_reads_it_back() {
        let fields = NoteFields {
            grade: "10th".into(),
            guardian_name: "Sam Smith".into(),
            guardian_phone: "(610) 555-1212".into(),
            guardian_email: String::new(),
            last_updated: date(2026, 11, 10),
            extra: "Line one\nLine two".into(),
        };
        let text = format(&fields);
        assert!(text.starts_with("Grade: 10th\r\nGuardian's Name: Sam Smith\r\n"));
        assert!(text.contains("Last updated: Tuesday, November 10, 2026"));
        assert_eq!(parse(&text), fields);
    }
}

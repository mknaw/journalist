use crate::entities::{Bullet, BulletType, Entry};
use anyhow::Result;
use chrono::NaiveDate;

pub struct MarkdownParser;

impl MarkdownParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, date: NaiveDate, content: &str) -> Result<Entry> {
        let mut entry = Entry::new(date);
        let mut current_bullet_type: Option<BulletType> = None;

        for line in content.lines() {
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            if line.starts_with('#') {
                current_bullet_type = match line.to_lowercase().as_str() {
                    "# tasks" => Some(BulletType::Task),
                    "# events" => Some(BulletType::Event),
                    "# notes" => Some(BulletType::Note),
                    "# priority" => Some(BulletType::Priority),
                    "# inspiration" => Some(BulletType::Inspiration),
                    "# insights" => Some(BulletType::Insight),
                    "# missteps" => Some(BulletType::Misstep),
                    _ => None,
                };
                continue;
            }

            if let Some(bullet_type) = current_bullet_type {
                let bullet = Bullet::new(line, bullet_type);
                entry.add_bullet(bullet);
            }
        }

        Ok(entry)
    }

    pub fn serialize(&self, entry: &Entry) -> Result<String> {
        let mut content = String::new();

        let sections = [
            (BulletType::Task, "# Tasks"),
            (BulletType::Event, "# Events"),
            (BulletType::Note, "# Notes"),
            (BulletType::Priority, "# Priority"),
            (BulletType::Inspiration, "# Inspiration"),
            (BulletType::Insight, "# Insights"),
            (BulletType::Misstep, "# Missteps"),
        ];

        for (bullet_type, section_header) in sections {
            let bullets = entry.get_bullets(&bullet_type);
            if !bullets.is_empty() {
                content.push_str(&format!("{}\n", section_header));
                for bullet in bullets {
                    content.push_str(&format!("{}\n", bullet.content));
                }
                content.push('\n');
            }
        }

        Ok(content)
    }

    /// Serialize entry for editing - always shows all headers for better UX
    pub fn serialize_for_editing(&self, entry: &Entry) -> Result<String> {
        let mut content = String::new();

        let sections = [
            (BulletType::Task, "# Tasks"),
            (BulletType::Event, "# Events"),
            (BulletType::Note, "# Notes"),
            (BulletType::Priority, "# Priority"),
            (BulletType::Inspiration, "# Inspiration"),
            (BulletType::Insight, "# Insights"),
            (BulletType::Misstep, "# Missteps"),
        ];

        for (bullet_type, section_header) in sections {
            let bullets = entry.get_bullets(&bullet_type);
            content.push_str(&format!("{}\n", section_header));
            for bullet in bullets {
                content.push_str(&format!("{}\n", bullet.content));
            }
            content.push('\n');
        }

        Ok(content)
    }

    /// Generate empty template for new entries
    pub fn empty_template() -> String {
        "# Tasks\n\n# Events\n\n# Notes\n\n# Priority\n\n# Inspiration\n\n# Insights\n\n# Missteps\n\n".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_empty_template_generation() {
        let template = MarkdownParser::empty_template();

        let expected = "# Tasks\n\n# Events\n\n# Notes\n\n# Priority\n\n# Inspiration\n\n# Insights\n\n# Missteps\n\n";
        assert_eq!(template, expected);

        // Verify template has all sections
        assert!(template.contains("# Tasks"));
        assert!(template.contains("# Events"));
        assert!(template.contains("# Notes"));
        assert!(template.contains("# Priority"));
        assert!(template.contains("# Inspiration"));
        assert!(template.contains("# Insights"));
        assert!(template.contains("# Missteps"));
    }

    #[test]
    fn test_serialize_empty_entry() {
        let parser = MarkdownParser::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        let entry = Entry::new(date);

        let result = parser.serialize(&entry).unwrap();

        // Empty entry should serialize to empty string (no sections with bullets)
        assert_eq!(result, "");
    }

    #[test]
    fn test_serialize_entry_with_bullets() {
        let parser = MarkdownParser::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        let mut entry = Entry::new(date);

        // Add various bullet types
        entry.add_bullet(Bullet::new("Complete project proposal", BulletType::Task));
        entry.add_bullet(Bullet::new("Team meeting at 2pm", BulletType::Event));
        entry.add_bullet(Bullet::new("New framework announced", BulletType::Note));
        entry.add_bullet(Bullet::new("Submit quarterly report", BulletType::Priority));

        let result = parser.serialize(&entry).unwrap();

        let expected = "# Tasks\nComplete project proposal\n\n# Events\nTeam meeting at 2pm\n\n# Notes\nNew framework announced\n\n# Priority\nSubmit quarterly report\n\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_template_creates_empty_entry() {
        let parser = MarkdownParser::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        let template = MarkdownParser::empty_template();

        let entry = parser.parse(date, &template).unwrap();

        assert_eq!(entry.date, date);
        assert_eq!(entry.total_bullets(), 0);

        // Verify all bullet types are empty
        assert!(entry.get_bullets(&BulletType::Task).is_empty());
        assert!(entry.get_bullets(&BulletType::Event).is_empty());
        assert!(entry.get_bullets(&BulletType::Note).is_empty());
        assert!(entry.get_bullets(&BulletType::Priority).is_empty());
        assert!(entry.get_bullets(&BulletType::Inspiration).is_empty());
        assert!(entry.get_bullets(&BulletType::Insight).is_empty());
        assert!(entry.get_bullets(&BulletType::Misstep).is_empty());
    }

    #[test]
    fn test_parse_entry_with_content() {
        let parser = MarkdownParser::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();

        let content = "# Tasks\nComplete project proposal\nReview code changes\n\n# Events\nTeam meeting at 2pm\n\n# Notes\nNew framework announced\n";

        let entry = parser.parse(date, content).unwrap();

        assert_eq!(entry.date, date);
        assert_eq!(entry.total_bullets(), 4);

        let tasks = entry.get_bullets(&BulletType::Task);
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].content, "Complete project proposal");
        assert_eq!(tasks[1].content, "Review code changes");

        let events = entry.get_bullets(&BulletType::Event);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].content, "Team meeting at 2pm");

        let notes = entry.get_bullets(&BulletType::Note);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].content, "New framework announced");
    }

    #[test]
    fn test_round_trip_consistency() {
        let parser = MarkdownParser::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        let mut original_entry = Entry::new(date);

        // Create entry with various bullets
        original_entry.add_bullet(Bullet::new("Task 1", BulletType::Task));
        original_entry.add_bullet(Bullet::new("Task 2", BulletType::Task));
        original_entry.add_bullet(Bullet::new("Event 1", BulletType::Event));
        original_entry.add_bullet(Bullet::new("Note 1", BulletType::Note));
        original_entry.add_bullet(Bullet::new("Priority item", BulletType::Priority));
        original_entry.add_bullet(Bullet::new("Great idea", BulletType::Inspiration));
        original_entry.add_bullet(Bullet::new("Learned something", BulletType::Insight));
        original_entry.add_bullet(Bullet::new("Made a mistake", BulletType::Misstep));

        // Serialize to markdown
        let markdown = parser.serialize(&original_entry).unwrap();

        // Parse back from markdown
        let parsed_entry = parser.parse(date, &markdown).unwrap();

        // Should be identical
        assert_eq!(original_entry.date, parsed_entry.date);
        assert_eq!(original_entry.total_bullets(), parsed_entry.total_bullets());

        // Check each bullet type
        for bullet_type in [
            BulletType::Task,
            BulletType::Event,
            BulletType::Note,
            BulletType::Priority,
            BulletType::Inspiration,
            BulletType::Insight,
            BulletType::Misstep,
        ] {
            let original_bullets = original_entry.get_bullets(&bullet_type);
            let parsed_bullets = parsed_entry.get_bullets(&bullet_type);

            assert_eq!(
                original_bullets.len(),
                parsed_bullets.len(),
                "Bullet count mismatch for {:?}",
                bullet_type
            );

            for (orig, parsed) in original_bullets.iter().zip(parsed_bullets.iter()) {
                assert_eq!(
                    orig.content, parsed.content,
                    "Content mismatch for {:?}",
                    bullet_type
                );
                assert_eq!(
                    orig.bullet_type, parsed.bullet_type,
                    "Type mismatch for {:?}",
                    bullet_type
                );
            }
        }
    }

    #[test]
    fn test_section_order_preservation() {
        let parser = MarkdownParser::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        let mut entry = Entry::new(date);

        // Add bullets in non-standard order
        entry.add_bullet(Bullet::new("Insight first", BulletType::Insight));
        entry.add_bullet(Bullet::new("Task second", BulletType::Task));
        entry.add_bullet(Bullet::new("Event third", BulletType::Event));

        let markdown = parser.serialize(&entry).unwrap();

        // Should follow the standard section order in serialization
        let lines: Vec<&str> = markdown.lines().collect();
        let task_line = lines.iter().position(|&line| line == "# Tasks").unwrap();
        let event_line = lines.iter().position(|&line| line == "# Events").unwrap();
        let insight_line = lines.iter().position(|&line| line == "# Insights").unwrap();

        // Tasks should come before Events, which should come before Insights
        assert!(task_line < event_line);
        assert!(event_line < insight_line);
    }

    #[test]
    fn test_serialize_for_editing_shows_all_headers() {
        let parser = MarkdownParser::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        let mut entry = Entry::new(date);

        // Add bullets to only some sections
        entry.add_bullet(Bullet::new("Only task", BulletType::Task));
        entry.add_bullet(Bullet::new("Only note", BulletType::Note));

        let result = parser.serialize_for_editing(&entry).unwrap();

        // Should include ALL headers even for empty sections
        assert!(result.contains("# Tasks"));
        assert!(result.contains("# Events"));
        assert!(result.contains("# Notes"));
        assert!(result.contains("# Priority"));
        assert!(result.contains("# Inspiration"));
        assert!(result.contains("# Insights"));
        assert!(result.contains("# Missteps"));

        // Should include the actual content too
        assert!(result.contains("Only task"));
        assert!(result.contains("Only note"));
    }

    #[test]
    fn test_whitespace_handling() {
        let parser = MarkdownParser::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();

        let content_with_whitespace =
            "  # Tasks  \n  Complete task  \n\n  # Events  \n  Meeting today  \n\n\n";

        let entry = parser.parse(date, content_with_whitespace).unwrap();

        let tasks = entry.get_bullets(&BulletType::Task);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].content, "Complete task");

        let events = entry.get_bullets(&BulletType::Event);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].content, "Meeting today");
    }

    #[test]
    fn test_case_insensitive_headers() {
        let parser = MarkdownParser::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();

        let content =
            "# TASKS\nUppercase header\n# events\nLowercase header\n# Notes\nMixed case\n";

        let entry = parser.parse(date, content).unwrap();

        assert_eq!(entry.get_bullets(&BulletType::Task).len(), 1);
        assert_eq!(entry.get_bullets(&BulletType::Event).len(), 1);
        assert_eq!(entry.get_bullets(&BulletType::Note).len(), 1);
    }
}

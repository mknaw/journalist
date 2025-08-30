use std::path::PathBuf;

pub struct Config {
    pub journal_dir: PathBuf,
    pub data_dir: PathBuf,
    pub indexes_dir: PathBuf,
    pub editor: String,
}

impl Config {
    pub fn from_env() -> Self {
        let journal_dir = std::env::var("JOURNAL_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::data_local_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("journo")
            });

        let data_dir = journal_dir.join("data");
        let indexes_dir = journal_dir.join("indexes");

        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());

        Self {
            journal_dir,
            data_dir,
            indexes_dir,
            editor,
        }
    }
}

/// Test utilities for DuckDB-based tests
///
/// This module provides a simple test harness that creates fresh DuckDB instances
/// for each test and automatically cleans them up. This ensures test isolation
/// without requiring complex rollback logic.
///
/// ## Usage Examples
///
/// ```rust
/// use crate::infrastructure::test_utils::test_harness::TestStorage;
///
/// #[test]
/// fn my_test() {
///     let test_storage = TestStorage::new();
///     let storage = test_storage.storage();
///     
///     // Use storage for testing...
///     // Database is automatically cleaned up when test_storage is dropped
/// }
///
/// // Or using the functional approach:
/// #[test]
/// fn my_test() {
///     test_harness::with_test_storage(|test_storage| {
///         // Use test_storage here...
///     });
/// }
/// ```
#[cfg(test)]
pub mod test_harness {
    use crate::entities::{Bullet, BulletType, Entry};
    use crate::infrastructure::DuckDbStorage;
    use anyhow::Result;
    use chrono::NaiveDate;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Test harness that creates a fresh DuckDB instance for each test
    /// and automatically cleans up when dropped
    pub struct TestStorage {
        pub storage: DuckDbStorage,
        _temp_dir: TempDir, // Keep temp dir alive
    }

    impl TestStorage {
        /// Create a new test storage instance with fresh DuckDB database
        pub fn new() -> Self {
            let temp_dir = TempDir::new().expect("Failed to create temp directory");
            let db_path = temp_dir.path().join("test.db");

            let storage =
                DuckDbStorage::new(&db_path).expect("Failed to initialize test DuckDB storage");

            Self {
                storage,
                _temp_dir: temp_dir,
            }
        }

        /// Get a reference to the DuckDB storage
        pub fn storage(&self) -> &DuckDbStorage {
            &self.storage
        }

        /// Get database path (useful for debugging)
        pub fn db_path(&self) -> PathBuf {
            self._temp_dir.path().join("test.db")
        }

        /// Convenience method to create a test entry with sample data
        pub fn create_sample_entry(&self, date: NaiveDate) -> Result<Entry> {
            use crate::infrastructure::storage::JournalStorage;

            let mut entry = Entry::new(date);
            entry.add_bullet(Bullet::new("Sample task", BulletType::Task));
            entry.add_bullet(Bullet::new("Sample event", BulletType::Event));
            entry.add_bullet(Bullet::new("Sample note", BulletType::Note));

            self.storage.save_entry(&entry)?;
            Ok(entry)
        }

        /// Convenience method to create and save a complex entry with all bullet types
        pub fn create_complex_entry(&self, date: NaiveDate) -> Result<Entry> {
            use crate::infrastructure::storage::JournalStorage;

            let mut entry = Entry::new(date);
            entry.add_bullet(Bullet::new("Task 1", BulletType::Task));
            entry.add_bullet(Bullet::new("Task 2", BulletType::Task));
            entry.add_bullet(Bullet::new("Important meeting", BulletType::Event));
            entry.add_bullet(Bullet::new("Remember this", BulletType::Note));
            entry.add_bullet(Bullet::new("High priority item", BulletType::Priority));
            entry.add_bullet(Bullet::new("Great idea", BulletType::Inspiration));
            entry.add_bullet(Bullet::new("Learned something", BulletType::Insight));
            entry.add_bullet(Bullet::new("Made a mistake", BulletType::Misstep));

            self.storage.save_entry(&entry)?;
            Ok(entry)
        }
    }

    impl Drop for TestStorage {
        fn drop(&mut self) {
            // TempDir automatically cleans up when dropped
            // No additional cleanup needed since DuckDB connection will close
        }
    }

    /// Convenience macro for creating test storage in tests
    #[macro_export]
    macro_rules! with_test_storage {
        ($storage_name:ident, $test_body:block) => {
            let $storage_name =
                $crate::infrastructure::test_utils::test_harness::TestStorage::new();
            $test_body
        };
    }

    /// Run a test with fresh test storage
    pub fn with_test_storage<F, R>(test_fn: F) -> R
    where
        F: FnOnce(&TestStorage) -> R,
    {
        let test_storage = TestStorage::new();
        test_fn(&test_storage)
    }
}

#[cfg(test)]
mod tests {
    use super::test_harness::*;
    use crate::infrastructure::storage::JournalStorage;
    use chrono::NaiveDate;

    #[test]
    fn test_harness_basic_functionality() {
        let test_storage = TestStorage::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();

        // Initially empty
        assert!(test_storage.storage().load_entry(date).unwrap().is_none());

        // Create sample entry
        let entry = test_storage.create_sample_entry(date).unwrap();
        assert_eq!(entry.total_bullets(), 3);

        // Verify it's persisted
        let loaded = test_storage.storage().load_entry(date).unwrap().unwrap();
        assert_eq!(loaded.total_bullets(), 3);
    }

    #[test]
    fn test_harness_with_function() {
        with_test_storage(|test_storage| {
            let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
            let complex_entry = test_storage.create_complex_entry(date).unwrap();
            assert_eq!(complex_entry.total_bullets(), 8);
        });
    }

    #[test]
    fn test_harness_isolation() {
        // Each test gets a fresh database
        let test_storage1 = TestStorage::new();
        let test_storage2 = TestStorage::new();

        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();

        // Add entry to first storage
        test_storage1.create_sample_entry(date).unwrap();

        // Second storage should not see it
        assert!(test_storage2.storage().load_entry(date).unwrap().is_none());
    }
}


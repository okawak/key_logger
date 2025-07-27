use crate::error::{KeyLoggerError, Result};
use chrono::Local;
use csv::WriterBuilder;
use std::{
    collections::HashMap,
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
};

pub fn export_to_csv_with_path(
    stats: &HashMap<&'static str, u64>,
    output_dir: Option<&Path>,
) -> Result<PathBuf> {
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let filename = format!("keylog_{timestamp}.csv");

    let file_path = if let Some(dir) = output_dir {
        std::fs::create_dir_all(dir).map_err(|e| KeyLoggerError::CreateDir {
            path: dir.to_path_buf(),
            source: e,
        })?;
        dir.join(&filename)
    } else {
        filename.into()
    };

    let file = File::create(&file_path).map_err(|e| KeyLoggerError::CreateFile {
        path: file_path.clone(),
        source: e,
    })?;

    let writer = BufWriter::new(file);
    #[allow(unused_mut)]
    let mut builder = WriterBuilder::new();
    #[cfg(windows)]
    {
        use csv::Terminator;
        builder.terminator(Terminator::CRLF);
    }

    let mut wtr = builder.from_writer(writer);

    wtr.write_record(["Key", "Count"])?;
    let mut rows: Vec<(&str, u64)> = stats.iter().map(|(&k, &v)| (k, v)).collect();
    rows.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));

    for (key, count) in rows {
        let count_s = count.to_string();
        wtr.write_record([key, count_s.as_str()])?;
    }

    wtr.flush()?;
    Ok(file_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[test]
    fn test_export_to_csv_basic() {
        let temp_dir = TempDir::new().unwrap();
        let mut stats = HashMap::new();
        stats.insert("A", 5);
        stats.insert("B", 3);
        stats.insert("Space", 10);

        let result = export_to_csv_with_path(&stats, Some(temp_dir.path()));
        assert!(result.is_ok());

        // Verify file exists
        assert!(std::path::Path::new(&result.unwrap()).exists());
    }

    #[test]
    fn test_export_to_csv_with_custom_path() {
        let temp_dir = TempDir::new().unwrap();
        let mut stats = HashMap::new();
        stats.insert("A", 1);

        let result = export_to_csv_with_path(&stats, Some(temp_dir.path()));
        assert!(result.is_ok());

        let filename = result.unwrap();
        assert!(std::path::Path::new(&filename).exists());
    }

    #[test]
    fn test_csv_content_format() {
        let temp_dir = TempDir::new().unwrap();
        let mut stats = HashMap::new();
        stats.insert("A", 5);
        stats.insert("B", 3);

        let result = export_to_csv_with_path(&stats, Some(temp_dir.path()));
        assert!(result.is_ok());

        let filename = result.unwrap();

        // Use csv crate to parse content in a platform-independent way
        let file = std::fs::File::open(&filename).unwrap();
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);

        let headers: Vec<String> = reader
            .headers()
            .unwrap()
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(headers, vec!["Key", "Count"]);

        let mut records: Vec<(String, u64)> = Vec::new();
        for result in reader.records() {
            let record = result.unwrap();
            let key = record[0].to_string();
            let count = record[1].parse::<u64>().unwrap();
            records.push((key, count));
        }

        // Check that all expected records are present
        assert!(records.iter().any(|(k, c)| k == "A" && *c == 5));
        assert!(records.iter().any(|(k, c)| k == "B" && *c == 3));

        // Check that records are sorted by count (descending)
        for i in 1..records.len() {
            let (prev_key, prev_count) = &records[i - 1];
            let (curr_key, curr_count) = &records[i];

            // If counts are different, previous should be >= current
            if prev_count != curr_count {
                assert!(
                    prev_count >= curr_count,
                    "Records not sorted by count: {prev_key} ({prev_count}) should come before {curr_key} ({curr_count})",
                );
            }
        }
    }

    #[test]
    fn test_csv_line_endings_cross_platform() {
        let temp_dir = TempDir::new().unwrap();
        let mut stats = HashMap::new();
        stats.insert("Enter", 10);
        stats.insert("Tab", 5);

        let result = export_to_csv_with_path(&stats, Some(temp_dir.path()));
        assert!(result.is_ok());

        let filename = result.unwrap();
        let content = std::fs::read_to_string(&filename).unwrap();

        // Verify that content contains expected data regardless of line endings
        assert!(content.contains("Key,Count"));
        assert!(content.contains("Enter,10"));
        assert!(content.contains("Tab,5"));

        // Test with CSV parser to ensure proper parsing regardless of line endings
        let file = std::fs::File::open(&filename).unwrap();
        let mut reader = csv::ReaderBuilder::new().from_reader(file);

        let record_count = reader.records().count();
        assert_eq!(record_count, 2); // Should have exactly 2 data records
    }

    #[test]
    fn test_empty_stats_export() {
        let temp_dir = TempDir::new().unwrap();
        let stats = HashMap::new();

        let result = export_to_csv_with_path(&stats, Some(temp_dir.path()));
        assert!(result.is_ok());

        let filename = result.unwrap();
        let file = std::fs::File::open(&filename).unwrap();
        let mut reader = csv::ReaderBuilder::new().from_reader(file);

        // Should still have headers
        let headers: Vec<String> = reader
            .headers()
            .unwrap()
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(headers, vec!["Key", "Count"]);

        // But no data records
        let record_count = reader.records().count();
        assert_eq!(record_count, 0);
    }

    #[test]
    fn test_large_counts() {
        let temp_dir = TempDir::new().unwrap();
        let mut stats = HashMap::new();
        stats.insert("Space", u64::MAX);
        stats.insert("A", 999_999_999);

        let result = export_to_csv_with_path(&stats, Some(temp_dir.path()));
        assert!(result.is_ok());

        let filename = result.unwrap();
        let file = std::fs::File::open(&filename).unwrap();
        let mut reader = csv::ReaderBuilder::new().from_reader(file);

        let mut found_max = false;
        let mut found_large = false;

        for result in reader.records() {
            let record = result.unwrap();
            let key = &record[0];
            let count = record[1].parse::<u64>().unwrap();

            if key == "Space" && count == u64::MAX {
                found_max = true;
            }
            if key == "A" && count == 999_999_999 {
                found_large = true;
            }
        }

        assert!(found_max, "Failed to find u64::MAX value");
        assert!(found_large, "Failed to find large count value");
    }
}

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
        let content = std::fs::read_to_string(&filename).unwrap();

        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines[0], "Key,Count");

        // Should be sorted by count (descending)
        assert!(lines.contains(&"A,5"));
        assert!(lines.contains(&"B,3"));

        // A should come before B (higher count)
        let a_pos = lines.iter().position(|&x| x == "A,5").unwrap();
        let b_pos = lines.iter().position(|&x| x == "B,3").unwrap();
        assert!(a_pos < b_pos);
    }
}

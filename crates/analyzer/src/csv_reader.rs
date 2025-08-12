use crate::error::{KbOptError, Result};
use crate::keys::{KeyId, ParseOptions, parse_key_label};

use csv::{ReaderBuilder, StringRecord, Trim};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

const EXPECTED_KEY_HEADER: &str = "Key";
const EXPECTED_COUNT_HEADER: &str = "Count";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KeyFreq {
    /// raw count of each key (only optimized keys) from data file
    raw_counts: HashMap<KeyId, u64>,
    /// total count
    total: u64,
}

impl KeyFreq {
    pub fn counts(&self) -> &HashMap<KeyId, u64> {
        &self.raw_counts
    }
    pub fn total(&self) -> u64 {
        self.total
    }

    /// Creates a new empty KeyFreq
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates KeyFreq from raw counts, automatically calculating total
    pub fn from_counts(raw_counts: HashMap<KeyId, u64>) -> Self {
        let total = raw_counts.values().copied().sum();
        Self { raw_counts, total }
    }

    /// Returns normalized probabilities for each key
    ///
    /// # Returns
    /// Empty HashMap if total is 0, otherwise probability for each key
    pub fn probabilities(&self) -> HashMap<KeyId, f64> {
        if self.total == 0 {
            return HashMap::new();
        }

        let denom = self.total as f64;
        self.raw_counts
            .iter()
            .map(|(&key, &count)| (key, count as f64 / denom))
            .collect()
    }

    /// Merges another KeyFreq into this one, combining counts
    ///
    /// # Arguments
    /// * `other` - KeyFreq to merge into this one
    pub fn merge(&mut self, other: KeyFreq) {
        for (k, v) in other.raw_counts {
            *self.raw_counts.entry(k).or_insert(0) += v;
        }
        // Recalculate total (in case the total changed independently)
        self.total = self.raw_counts.values().copied().sum();
    }

    /// Returns the count for a specific key
    pub fn get_count(&self, key: KeyId) -> u64 {
        self.raw_counts.get(&key).copied().unwrap_or(0)
    }

    /// Returns true if no keys have been recorded
    pub fn is_empty(&self) -> bool {
        self.raw_counts.is_empty()
    }

    /// Returns the number of unique keys
    pub fn unique_keys(&self) -> usize {
        self.raw_counts.len()
    }

    /// Converts KeyFreq to the HashMap<String, u64> format used by the optimizer
    pub fn to_optimizer_format(&self) -> std::collections::HashMap<String, u64> {
        self.raw_counts
            .iter()
            .map(|(key_id, &count)| (key_id.to_string(), count))
            .collect()
    }
}

/// Reads key frequency data from a CSV file
///
/// # Arguments
/// * `path` - Path to the CSV file
/// * `parse_options` - Options for parsing key labels
///
/// # Errors
/// Returns error if file cannot be read or CSV format is invalid
pub fn read_key_freq_csv<P: AsRef<Path>>(path: P, opt: &ParseOptions) -> Result<KeyFreq> {
    let file = std::fs::File::open(path)?;
    read_key_freq_from_reader(file, opt)
}

/// Reads and merges all CSV files from a directory
///
/// # Arguments
/// * `dir_path` - Path to directory containing CSV files
/// * `parse_options` - Options for parsing key labels
///
/// # Returns
/// Merged KeyFreq from all CSV files in the directory
///
/// # Errors
/// Returns error if directory cannot be read or no valid CSV files found
pub fn read_key_freq_from_directory<P: AsRef<Path>>(dir_path: P, opt: &ParseOptions) -> Result<KeyFreq> {
    let dir_path = dir_path.as_ref();
    
    if !dir_path.exists() {
        return Err(KbOptError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Directory does not exist: {}", dir_path.display()),
        )));
    }
    
    if !dir_path.is_dir() {
        return Err(KbOptError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Path is not a directory: {}", dir_path.display()),
        )));
    }

    let mut merged_freq = KeyFreq::new();
    let mut csv_files_processed = 0;

    // Read directory entries
    for entry in std::fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        
        // Skip directories and non-CSV files
        if !path.is_file() {
            continue;
        }
        
        if let Some(extension) = path.extension() {
            if extension.to_str() != Some("csv") {
                continue;
            }
        } else {
            continue;
        }

        // Try to read the CSV file
        match read_key_freq_csv(&path, opt) {
            Ok(freq) => {
                if !freq.is_empty() {
                    merged_freq.merge(freq);
                    csv_files_processed += 1;
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to read CSV file {}: {}", path.display(), e);
                // Continue processing other files instead of failing
            }
        }
    }

    if csv_files_processed == 0 {
        return Err(KbOptError::Other(format!(
            "No valid CSV files found in directory: {}", 
            dir_path.display()
        )));
    }

    Ok(merged_freq)
}

/// Read CSV with `Key,Count` format.
/// - Character keys (A..Z) are automatically ignored by parse_key_label
/// - F/Numpad/Nav keys are included only if enabled in ParseOptions
/// - If strict_unknown_keys=true, an error is raised for unknown labels
pub fn read_key_freq_from_reader<R: Read>(reader: R, opt: &ParseOptions) -> Result<KeyFreq> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .trim(Trim::All)
        .flexible(true) // allow additional columns
        .from_reader(reader);

    validate_csv_headers(&mut rdr)?;

    let mut counts: HashMap<KeyId, u64> = HashMap::new();

    for (i, result) in rdr.records().enumerate() {
        let rec = result?;
        let row = i + 2; // CSV rows are 1-indexed, +1 for header

        if let Some((kid, n)) = parse_record(&rec, row, opt)? {
            *counts.entry(kid).or_insert(0) += n;
        }
    }

    Ok(KeyFreq::from_counts(counts))
}

/// Validates CSV headers match expected format
fn validate_csv_headers<R: Read>(csv_reader: &mut csv::Reader<R>) -> Result<()> {
    let headers = csv_reader
        .headers()
        .map_err(|e| KbOptError::CsvHeader(format!("Failed to read headers: {}", e)))?;

    let key_header = headers
        .get(0)
        .ok_or_else(|| KbOptError::CsvHeader("Missing key column at index 0".to_string()))?;

    let count_header = headers
        .get(1)
        .ok_or_else(|| KbOptError::CsvHeader("Missing count column at index 1".to_string()))?;

    if !key_header.eq_ignore_ascii_case(EXPECTED_KEY_HEADER) {
        return Err(KbOptError::CsvHeader(format!(
            "Expected '{}' in column 0, found '{}'",
            EXPECTED_KEY_HEADER, key_header
        )));
    }

    if !count_header.eq_ignore_ascii_case(EXPECTED_COUNT_HEADER) {
        return Err(KbOptError::CsvHeader(format!(
            "Expected '{}' in column 1, found '{}'",
            EXPECTED_COUNT_HEADER, count_header
        )));
    }

    Ok(())
}

fn parse_record(
    rec: &StringRecord,
    row: usize,
    opt: &ParseOptions,
) -> Result<Option<(KeyId, u64)>> {
    if rec.iter().all(|f| f.trim().is_empty()) {
        return Ok(None);
    }
    let key_label = get_column_value(rec, 0, row)?;
    let count_str = get_column_value(rec, 1, row)?;

    if key_label.is_empty() {
        return Ok(None);
    }

    match parse_key_label(key_label, opt) {
        Some(kid) => {
            let count = parse_count_value(count_str, row)?;
            Ok(Some((kid, count)))
        }
        None if opt.strict_unknown_keys => Err(KbOptError::UnknownKey {
            row,
            label: key_label.to_string(),
        }),
        None => Ok(None),
    }
}

/// Safely extracts a column value from a CSV record
fn get_column_value(record: &StringRecord, column_index: usize, row_number: usize) -> Result<&str> {
    record
        .get(column_index)
        .map(str::trim)
        .ok_or_else(|| KbOptError::CsvRow {
            row: row_number,
            got: record.len(),
        })
}

/// Parses a count string into u64
fn parse_count_value(count_str: &str, row_number: usize) -> Result<u64> {
    count_str
        .parse()
        .map_err(|parse_error| KbOptError::CountParse {
            row: row_number,
            value: count_str.to_string(),
            source: parse_error,
        })
}

#[cfg(test)]
mod directory_tests {
    use super::*;
    use crate::keys::{KeyId, ParseOptions};

    #[test]
    fn test_read_key_freq_from_directory_nonexistent() {
        use std::path::PathBuf;
        let opt = ParseOptions::default();
        let nonexistent = PathBuf::from("nonexistent_directory");

        let result = read_key_freq_from_directory(&nonexistent, &opt);
        assert!(result.is_err());
    }

    #[test]
    fn test_key_freq_merge() {
        use std::collections::HashMap;

        let mut freq1_counts = HashMap::new();
        freq1_counts.insert(KeyId::Tab, 10);
        freq1_counts.insert(KeyId::Space, 20);
        let mut freq1 = KeyFreq::from_counts(freq1_counts);

        let mut freq2_counts = HashMap::new();
        freq2_counts.insert(KeyId::Tab, 5); // Should merge with freq1
        freq2_counts.insert(KeyId::Enter, 15);
        let freq2 = KeyFreq::from_counts(freq2_counts);

        freq1.merge(freq2);

        assert_eq!(freq1.get_count(KeyId::Tab), 15); // 10 + 5
        assert_eq!(freq1.get_count(KeyId::Space), 20);
        assert_eq!(freq1.get_count(KeyId::Enter), 15);
        assert_eq!(freq1.total(), 50); // 15 + 20 + 15
        assert_eq!(freq1.unique_keys(), 3);
    }

    #[test]
    fn test_to_optimizer_format() {
        use std::collections::HashMap;

        let mut counts = HashMap::new();
        counts.insert(KeyId::Tab, 100);
        counts.insert(KeyId::Space, 200);
        counts.insert(KeyId::Digit(1), 50);
        
        let freq = KeyFreq::from_counts(counts);
        let optimizer_format = freq.to_optimizer_format();

        assert_eq!(optimizer_format.get("Tab"), Some(&100));
        assert_eq!(optimizer_format.get("Space"), Some(&200));
        assert_eq!(optimizer_format.get("1"), Some(&50));
        assert_eq!(optimizer_format.len(), 3);
    }
}

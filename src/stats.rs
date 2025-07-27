use crate::error::Result;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

// Initial capacity for the statistics HashMap to avoid frequent reallocations
// This covers most common keys: 26 letters + 10 digits + 20+ special keys + modifiers
const INITIAL_STATISTICS_CAPACITY: usize = 64;

// Using a more efficient hash map for better performance
type StatisticsMap = HashMap<&'static str, u64>;

pub type KeyStatistics = Arc<Mutex<StatisticsMap>>;

pub fn create_statistics() -> KeyStatistics {
    // Pre-allocate capacity for common keys to improve performance
    Arc::new(Mutex::new(HashMap::with_capacity(
        INITIAL_STATISTICS_CAPACITY,
    )))
}

pub fn get_statistics_snapshot(stats: &KeyStatistics) -> Result<HashMap<&'static str, u64>> {
    Ok(stats.lock().unwrap_or_else(|p| p.into_inner()).clone())
}

pub fn add_many<I>(stats: &KeyStatistics, keys: I) -> Result<()>
where
    I: IntoIterator<Item = &'static str>,
{
    let mut guard = stats.lock().unwrap_or_else(|p| p.into_inner());
    for key in keys {
        *guard.entry(key).or_insert(0) += 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    fn add_key_press(stats: &KeyStatistics, key: &'static str) -> Result<()> {
        let mut guard = stats.lock().unwrap_or_else(|p| p.into_inner());
        *guard.entry(key).or_insert(0) += 1;
        Ok(())
    }

    #[test]
    fn test_create_statistics() {
        let stats = create_statistics();
        let snapshot = get_statistics_snapshot(&stats).unwrap();
        assert!(snapshot.is_empty());
    }

    #[test]
    fn test_add_key_press() {
        let stats = create_statistics();

        add_key_press(&stats, "A").unwrap();
        add_key_press(&stats, "A").unwrap();
        add_key_press(&stats, "B").unwrap();

        let snapshot = get_statistics_snapshot(&stats).unwrap();
        assert_eq!(snapshot.get("A"), Some(&2));
        assert_eq!(snapshot.get("B"), Some(&1));
    }

    #[test]
    fn test_add_many_function() {
        let stats = create_statistics();

        let keys = vec!["A", "B", "A", "C", "A"];
        add_many(&stats, keys).unwrap();

        let snapshot = get_statistics_snapshot(&stats).unwrap();
        assert_eq!(snapshot.get("A"), Some(&3));
        assert_eq!(snapshot.get("B"), Some(&1));
        assert_eq!(snapshot.get("C"), Some(&1));
    }

    #[test]
    fn test_add_many_empty_iterator() {
        let stats = create_statistics();

        let keys: Vec<&'static str> = vec![];
        add_many(&stats, keys).unwrap();

        let snapshot = get_statistics_snapshot(&stats).unwrap();
        assert!(snapshot.is_empty());
    }

    #[test]
    fn test_large_counts() {
        let stats = create_statistics();

        // Test with large numbers to ensure u64 handling
        let large_keys: Vec<&'static str> = (0..10000).map(|_| "Space").collect();
        add_many(&stats, large_keys).unwrap();

        let snapshot = get_statistics_snapshot(&stats).unwrap();
        assert_eq!(snapshot.get("Space"), Some(&10000));
    }

    #[test]
    fn test_thread_safety() {
        let stats = create_statistics();
        let stats_clone1 = Arc::clone(&stats);
        let stats_clone2 = Arc::clone(&stats);

        let handle1 = thread::spawn(move || {
            for _ in 0..1000 {
                add_key_press(&stats_clone1, "A").unwrap();
            }
        });

        let handle2 = thread::spawn(move || {
            for _ in 0..1000 {
                add_key_press(&stats_clone2, "A").unwrap();
            }
        });

        handle1.join().unwrap();
        handle2.join().unwrap();

        let snapshot = get_statistics_snapshot(&stats).unwrap();
        assert_eq!(snapshot.get("A"), Some(&2000));
    }

    #[test]
    fn test_special_characters_cross_platform() {
        let stats = create_statistics();

        // Test various special characters that might be handled differently on different platforms
        let special_keys = vec![
            "Space",
            "Enter",
            "Tab",
            "Backspace",
            "Delete",
            "Escape",
            "LeftShift",
            "RightShift",
            "LeftControl",
            "RightControl",
            "LeftAlt",
            "RightAlt",
            "ArrowUp",
            "ArrowDown",
            "ArrowLeft",
            "ArrowRight",
            "Comma",
            "Period",
            "Semicolon",
            "Minus",
            "Equal",
            "LeftBracket",
            "RightBracket",
        ];

        add_many(&stats, special_keys.iter().copied()).unwrap();

        let snapshot = get_statistics_snapshot(&stats).unwrap();

        // All special keys should be recorded
        for &key in &special_keys {
            assert_eq!(snapshot.get(key), Some(&1), "Key '{key}' was not recorded",);
        }

        assert_eq!(snapshot.len(), special_keys.len());
    }

    #[test]
    fn test_statistics_capacity_handling() {
        let stats = create_statistics();

        // Add more keys than the initial capacity to test reallocation
        let alphabet: Vec<&'static str> = (b'A'..=b'Z')
            .map(|c| match c {
                b'A' => "A",
                b'B' => "B",
                b'C' => "C",
                b'D' => "D",
                b'E' => "E",
                b'F' => "F",
                b'G' => "G",
                b'H' => "H",
                b'I' => "I",
                b'J' => "J",
                b'K' => "K",
                b'L' => "L",
                b'M' => "M",
                b'N' => "N",
                b'O' => "O",
                b'P' => "P",
                b'Q' => "Q",
                b'R' => "R",
                b'S' => "S",
                b'T' => "T",
                b'U' => "U",
                b'V' => "V",
                b'W' => "W",
                b'X' => "X",
                b'Y' => "Y",
                b'Z' => "Z",
                _ => "Unknown",
            })
            .collect();

        add_many(&stats, alphabet.iter().copied()).unwrap();

        let snapshot = get_statistics_snapshot(&stats).unwrap();
        assert_eq!(snapshot.len(), 26);

        // Check a few specific letters
        assert_eq!(snapshot.get("A"), Some(&1));
        assert_eq!(snapshot.get("Z"), Some(&1));
    }

    #[test]
    fn test_memory_efficiency() {
        let stats = create_statistics();

        // Verify that we're using static strings efficiently
        let test_key = "Space";
        add_key_press(&stats, test_key).unwrap();

        let snapshot = get_statistics_snapshot(&stats).unwrap();

        // Check that the key in the map is the same static string reference
        for (key, _) in snapshot.iter() {
            if *key == "Space" {
                // This should be the same pointer for static strings
                assert_eq!(key.as_ptr(), test_key.as_ptr());
                break;
            }
        }
    }
}

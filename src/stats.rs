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
}

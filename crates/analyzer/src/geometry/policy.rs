use crate::keys::KeyId;
use std::collections::HashMap;

/// Arrow allowed band (row・x range half-open interval [x0, x1))
#[derive(Debug, Clone)]
pub struct ArrowBand {
    pub row: usize,
    pub x0_u: f32,
    pub x1_u: f32,
}

/// Placement policy
#[derive(Debug, Clone, Default)]
pub struct Policy {
    /// "Allowed rows" for each general key (empty = all rows allowed)
    pub allowed_rows: HashMap<KeyId, Vec<usize>>,
    /// Arrow allowed bands (multiple rows OK)
    pub arrow_bands: Vec<ArrowBand>,
}

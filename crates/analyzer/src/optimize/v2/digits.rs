// Phase 4: 数値クラスターの実装準備
// このファイルは Phase 4 実装時に詳細化される

use crate::error::KbOptError;

/// 数値クラスターの設定
#[derive(Debug, Clone)]
pub struct DigitClusterConfig {
    pub enable_digit_cluster: bool,
    pub enforce_order: bool,      // 順序制約を強制するか
    pub allowed_rows: Vec<usize>, // 数字配置可能行
}

impl Default for DigitClusterConfig {
    fn default() -> Self {
        Self {
            enable_digit_cluster: false,
            enforce_order: true,
            allowed_rows: vec![0, 1], // 上段、ファンクション行など
        }
    }
}

/// Phase 4で実装予定: 数値順序制約の追加
pub fn add_digit_ordering_constraints<M>(
    _model: &mut M,
    _config: &DigitClusterConfig,
) -> Result<(), KbOptError>
where
    M: good_lp::SolverModel,
{
    // Phase 4で実装
    todo!("Phase 4: digit ordering constraints not yet implemented")
}

/// Phase 4で実装予定: 数値クラスターの目的関数項追加
pub fn add_digit_objective_terms(
    _objective: &mut good_lp::Expression,
    _config: &DigitClusterConfig,
) {
    // Phase 4で実装
    todo!("Phase 4: digit cluster objective terms not yet implemented")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digit_cluster_config_default() {
        let config = DigitClusterConfig::default();
        assert!(!config.enable_digit_cluster);
        assert!(config.enforce_order);
        assert_eq!(config.allowed_rows, vec![0, 1]);
    }
}

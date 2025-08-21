use crate::error::KbOptError;
use crate::geometry::types::Finger;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// メイン設定構造体
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub solver: SolverConfig,
    pub v1: V1Config,
    pub v2: V2Config,
    #[serde(default)]
    pub comparison: Option<ComparisonConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SolverConfig {
    pub version: String, // "v1" | "v2"
    pub output_dir: String,
    pub geometry: String, // "row-stagger" | "ortho" | "column-stagger"
    pub csv_dir: String,
    pub max_rows: Option<usize>, // geometryの行数（デフォルト6）
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct V1Config {
    pub include_fkeys: bool,
    pub a_ms: f64,
    pub b_ms: f64,
    /// 指別Fitts係数の有効化
    #[serde(default)]
    pub enable_fingerwise_fitts: bool,
    /// 数字クラスタの有効化
    #[serde(default)]
    pub enable_digit_cluster: bool,
    /// 方向依存幅の有効化
    #[serde(default)]
    pub enable_directional_width: bool,
    /// 最適化重み設定
    #[serde(default)]
    pub weights: OptimizationWeightsConfig,
    /// 指別Fitts係数設定
    #[serde(default)]
    pub fingerwise_coeffs: Option<HashMap<String, FittsCoeffConfig>>,
    /// 数字クラスタ設定
    #[serde(default)]
    pub digit_cluster: Option<DigitClusterConfig>,
    /// 行位置の自由化設定
    #[serde(default)]
    pub row_flexibility: Option<RowFlexibilityConfigFile>,
    /// 最適化変数の詳細設定
    #[serde(default)]
    pub optimization_vars: Option<OptimizationVarsConfigFile>,
    /// ソルバー定数設定
    #[serde(default)]
    pub solver_constants: Option<SolverConstantsConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct OptimizationWeightsConfig {
    pub normal_keys: f64,
    pub arrow_and_digit_keys: f64,
    pub width_penalty: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FittsCoeffConfig {
    pub a_ms: f64,
    pub b_ms: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DigitClusterConfig {
    pub enable: bool,
    pub enforce_sequence: bool,
    pub allowed_rows: Vec<usize>,
    pub enforce_horizontal: bool,
    /// 左端揃え - 全ての行の左端位置を一致させる
    #[serde(default)]
    pub align_left_edge: bool,
    /// 右端揃え - 全ての行の右端位置を一致させる
    #[serde(default)]
    pub align_right_edge: bool,
}

/// 行位置の自由化設定（設定ファイル形式）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RowFlexibilityConfigFile {
    pub enable_free_positioning: bool,
    pub fixed_alphabet_rows: bool,
    pub optimizable_symbols: bool,
    pub min_rows_from_home: usize,
    pub max_rows_from_home: usize,
}

/// 最適化変数の詳細設定（設定ファイル形式）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OptimizationVarsConfigFile {
    pub auto_tune_weights: bool,
    pub use_frequency_scaling: bool,
    pub enable_bigram_penalty: bool,
    pub bigram_weight: f64,
    pub distance_penalty_factor: f64,
    pub finger_balance_weight: f64,
}

/// ソルバー定数設定（設定ファイル形式）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SolverConstantsConfig {
    /// 矢印キー関連定数
    pub required_arrow_blocks: usize,
    pub max_flow_per_block: f64,
    /// 数字クラスター関連定数
    pub required_digit_blocks: usize,
    pub max_digit_flow_per_block: f64,
    /// フロー関連定数
    pub flow_roots: f64,
    pub digit_flow_roots: f64,
    /// 解析閾値
    pub solution_threshold: f64,
}

impl Default for SolverConstantsConfig {
    fn default() -> Self {
        Self {
            required_arrow_blocks: 4,
            max_flow_per_block: 3.0,
            required_digit_blocks: 10,
            max_digit_flow_per_block: 9.0,
            flow_roots: 1.0,
            digit_flow_roots: 1.0,
            solution_threshold: 0.5,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct V2Config {
    #[serde(default)]
    pub fitts_coefficients: Option<FittsCoefficientsConfig>,
    #[serde(default)]
    pub directional_width: Option<DirectionalWidthConfig>,
    #[serde(default)]
    pub layers: Option<LayersConfig>,
    #[serde(default)]
    pub digit_cluster: Option<DigitClusterConfig>,
    #[serde(default)]
    pub bigrams: Option<BigramsConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FittsCoefficientsConfig {
    pub enable: bool,
    #[serde(default)]
    pub values: Option<HashMap<String, [f64; 2]>>, // finger -> [a_ms, b_ms]
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DirectionalWidthConfig {
    pub enable: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LayersConfig {
    pub enable: bool,
    pub modifier_penalty_ms: f64,
    pub modifier_rows: Vec<usize>,
    pub max_layers: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BigramsConfig {
    pub enable: bool,
    pub approach: String, // "DirectionalBucket" | "TopMLinearization"
    pub min_frequency: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ComparisonConfig {
    pub enable_parallel: bool,
    pub save_report: bool,
    pub report_format: String, // "html" | "json" | "csv"
}

impl Default for Config {
    fn default() -> Self {
        Self {
            solver: SolverConfig {
                version: "v1".to_string(),
                output_dir: "figs".to_string(),
                geometry: "row-stagger".to_string(),
                csv_dir: "csv".to_string(),
                max_rows: Some(6),
            },
            v1: V1Config {
                include_fkeys: false,
                a_ms: 0.0,
                b_ms: 1.0,
                enable_fingerwise_fitts: true,
                enable_digit_cluster: true,
                enable_directional_width: true,
                weights: OptimizationWeightsConfig {
                    normal_keys: 1.0,
                    arrow_and_digit_keys: 1.0,
                    width_penalty: 0.05,
                },
                fingerwise_coeffs: None,
                digit_cluster: None,
                row_flexibility: None,
                optimization_vars: None,
                solver_constants: None,
            },
            v2: V2Config {
                fitts_coefficients: Some(FittsCoefficientsConfig {
                    enable: false,
                    values: None,
                }),
                directional_width: Some(DirectionalWidthConfig { enable: false }),
                layers: Some(LayersConfig {
                    enable: false,
                    modifier_penalty_ms: 10.0,
                    modifier_rows: vec![3, 4],
                    max_layers: Some(1),
                }),
                digit_cluster: Some(DigitClusterConfig {
                    enable: false,
                    enforce_sequence: true,
                    allowed_rows: vec![0, 1],
                    enforce_horizontal: false,
                    align_left_edge: false,
                    align_right_edge: false,
                }),
                bigrams: Some(BigramsConfig {
                    enable: false,
                    approach: "DirectionalBucket".to_string(),
                    min_frequency: 10.0,
                }),
            },
            comparison: Some(ComparisonConfig {
                enable_parallel: true,
                save_report: true,
                report_format: "html".to_string(),
            }),
        }
    }
}

impl Config {
    /// 設定ファイルから読み込み
    pub fn load_from_file(path: &str) -> Result<Self, KbOptError> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            KbOptError::ConfigError(format!("Failed to read config file '{}': {}", path, e))
        })?;

        let config: Config = toml::from_str(&content).map_err(|e| {
            KbOptError::ConfigError(format!("Failed to parse config file '{}': {}", path, e))
        })?;

        config.validate()?;
        Ok(config)
    }

    /// 設定の検証
    pub fn validate(&self) -> Result<(), KbOptError> {
        // バージョンの検証
        match self.solver.version.as_str() {
            "v1" | "v2" => {}
            _ => {
                return Err(KbOptError::ConfigError(format!(
                    "Invalid solver version: {}. Must be 'v1' or 'v2'",
                    self.solver.version
                )));
            }
        }

        // ジオメトリの検証
        match self.solver.geometry.as_str() {
            "row-stagger" | "ortho" | "column-stagger" => {}
            _ => {
                return Err(KbOptError::ConfigError(format!(
                    "Invalid geometry: {}. Must be 'row-stagger', 'ortho', or 'column-stagger'",
                    self.solver.geometry
                )));
            }
        }

        // v1設定の検証
        if self.v1.b_ms <= 0.0 {
            return Err(KbOptError::ConfigError(
                "v1.b_ms must be positive".to_string(),
            ));
        }

        // v1のFitts係数検証
        self.validate_v1_fitts_coefficients()?;

        // v2設定の検証
        self.validate_fitts_coefficients()?;
        self.validate_bigrams_config()?;

        Ok(())
    }

    /// Fitts係数の検証
    fn validate_fitts_coefficients(&self) -> Result<(), KbOptError> {
        if let Some(ref fitts_config) = self.v2.fitts_coefficients
            && fitts_config.enable
            && let Some(ref values) = fitts_config.values
        {
            for (finger, coeffs) in values {
                if coeffs[1] <= 0.0 {
                    return Err(KbOptError::ConfigError(format!(
                        "Fitts coefficient b_ms for finger {} must be positive",
                        finger
                    )));
                }
            }
        }
        Ok(())
    }

    /// ビグラム設定の検証
    fn validate_bigrams_config(&self) -> Result<(), KbOptError> {
        if let Some(ref bigrams_config) = self.v2.bigrams
            && bigrams_config.enable
        {
            match bigrams_config.approach.as_str() {
                "DirectionalBucket" | "TopMLinearization" => {}
                _ => {
                    return Err(KbOptError::ConfigError(format!(
                        "Invalid bigrams approach: {}. Must be 'DirectionalBucket' or 'TopMLinearization'",
                        bigrams_config.approach
                    )));
                }
            }
        }
        Ok(())
    }

    /// v1のFitts係数の検証
    fn validate_v1_fitts_coefficients(&self) -> Result<(), KbOptError> {
        if self.v1.enable_fingerwise_fitts
            && let Some(ref coeffs_config) = self.v1.fingerwise_coeffs
        {
            for (finger, coeff_config) in coeffs_config {
                if coeff_config.b_ms <= 0.0 {
                    return Err(KbOptError::ConfigError(format!(
                        "v1 Fitts coefficient b_ms for finger {} must be positive",
                        finger
                    )));
                }
            }
        }
        Ok(())
    }

    /// v1のSolveOptionsに変換
    pub fn to_solve_options_v1(&self) -> crate::optimize::SolveOptions {
        crate::optimize::SolveOptions {
            include_fkeys: self.v1.include_fkeys,
            a_ms: self.v1.a_ms,
            b_ms: self.v1.b_ms,
        }
    }

    /// v1のOptionsに変換
    pub fn to_v1_options(&self) -> crate::optimize::v1::Options {
        // 指別Fitts係数の変換
        let mut fingerwise_coeffs = crate::optimize::FingerwiseFittsCoefficients {
            enable_directional_width: self.v1.enable_directional_width,
            ..Default::default()
        };

        if let Some(ref coeffs_config) = self.v1.fingerwise_coeffs {
            for (finger_str, coeff_config) in coeffs_config {
                if let Some(finger) = finger_from_string(finger_str) {
                    fingerwise_coeffs
                        .coefficients
                        .insert(finger, (coeff_config.a_ms, coeff_config.b_ms));
                }
            }
        }

        // クラスタ設定の変換
        let cluster_config = if let Some(ref digit_config) = self.v1.digit_cluster {
            crate::keys::ClusterConfig {
                enable_arrows: true, // 矢印は常に有効
                enable_digits: digit_config.enable,
                enforce_digit_sequence: digit_config.enforce_sequence,
                allowed_rows: digit_config.allowed_rows.clone(),
                enforce_horizontal: digit_config.enforce_horizontal,
                align_left_edge: digit_config.align_left_edge,
                align_right_edge: digit_config.align_right_edge,
            }
        } else {
            crate::keys::ClusterConfig::default()
        };

        // 最適化重み設定の変換
        let weights = crate::optimize::v1::OptimizationWeights {
            normal_keys: self.v1.weights.normal_keys,
            arrow_and_digit_keys: self.v1.weights.arrow_and_digit_keys,
            width_penalty: self.v1.weights.width_penalty,
        };

        // 行位置の自由化設定の変換
        let row_flexibility = if let Some(ref row_config) = self.v1.row_flexibility {
            crate::optimize::v1::RowFlexibilityConfig {
                enable_free_positioning: row_config.enable_free_positioning,
                fixed_alphabet_rows: row_config.fixed_alphabet_rows,
                optimizable_symbols: row_config.optimizable_symbols,
                min_rows_from_home: row_config.min_rows_from_home,
                max_rows_from_home: row_config.max_rows_from_home,
            }
        } else {
            crate::optimize::v1::RowFlexibilityConfig {
                enable_free_positioning: false,
                fixed_alphabet_rows: true,
                optimizable_symbols: true,
                min_rows_from_home: 1,
                max_rows_from_home: 2,
            }
        };

        // 最適化変数の詳細設定の変換
        let optimization_vars = if let Some(ref vars_config) = self.v1.optimization_vars {
            crate::optimize::v1::OptimizationVarsConfig {
                auto_tune_weights: vars_config.auto_tune_weights,
                use_frequency_scaling: vars_config.use_frequency_scaling,
                enable_bigram_penalty: vars_config.enable_bigram_penalty,
                bigram_weight: vars_config.bigram_weight,
                distance_penalty_factor: vars_config.distance_penalty_factor,
                finger_balance_weight: vars_config.finger_balance_weight,
            }
        } else {
            crate::optimize::v1::OptimizationVarsConfig {
                auto_tune_weights: false,
                use_frequency_scaling: true,
                enable_bigram_penalty: false,
                bigram_weight: 0.1,
                distance_penalty_factor: 1.0,
                finger_balance_weight: 0.0,
            }
        };

        // ソルバー定数の変換
        let solver_constants = if let Some(ref constants_config) = self.v1.solver_constants {
            crate::optimize::v1::SolverConstants {
                required_arrow_blocks: constants_config.required_arrow_blocks,
                max_flow_per_block: constants_config.max_flow_per_block,
                required_digit_blocks: constants_config.required_digit_blocks,
                max_digit_flow_per_block: constants_config.max_digit_flow_per_block,
                flow_roots: constants_config.flow_roots,
                digit_flow_roots: constants_config.digit_flow_roots,
                solution_threshold: constants_config.solution_threshold,
            }
        } else {
            crate::optimize::v1::SolverConstants::default()
        };

        crate::optimize::v1::Options {
            enable_fingerwise_fitts: self.v1.enable_fingerwise_fitts,
            enable_digit_cluster: self.v1.enable_digit_cluster,
            enable_directional_width: self.v1.enable_directional_width,
            fingerwise_coeffs,
            cluster_config,
            weights,
            row_flexibility,
            optimization_vars,
            solver_constants,
        }
    }
}

/// Finger enum と文字列の変換ユーティリティ
pub fn finger_from_string(s: &str) -> Option<Finger> {
    match s {
        "LThumb" => Some(Finger::LThumb),
        "LIndex" => Some(Finger::LIndex),
        "LMiddle" => Some(Finger::LMiddle),
        "LRing" => Some(Finger::LRing),
        "LPinky" => Some(Finger::LPinky),
        "RThumb" => Some(Finger::RThumb),
        "RIndex" => Some(Finger::RIndex),
        "RMiddle" => Some(Finger::RMiddle),
        "RRing" => Some(Finger::RRing),
        "RPinky" => Some(Finger::RPinky),
        _ => None,
    }
}

pub fn finger_to_string(finger: Finger) -> &'static str {
    match finger {
        Finger::LThumb => "LThumb",
        Finger::LIndex => "LIndex",
        Finger::LMiddle => "LMiddle",
        Finger::LRing => "LRing",
        Finger::LPinky => "LPinky",
        Finger::RThumb => "RThumb",
        Finger::RIndex => "RIndex",
        Finger::RMiddle => "RMiddle",
        Finger::RRing => "RRing",
        Finger::RPinky => "RPinky",
    }
}

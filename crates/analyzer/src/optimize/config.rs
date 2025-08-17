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
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct V1Config {
    pub include_fkeys: bool,
    pub a_ms: f64,
    pub b_ms: f64,
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
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DigitClusterConfig {
    pub enable: bool,
    pub enforce_order: bool,
    pub allowed_rows: Vec<usize>,
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
            },
            v1: V1Config {
                include_fkeys: false,
                a_ms: 0.0,
                b_ms: 1.0,
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
                }),
                digit_cluster: Some(DigitClusterConfig {
                    enable: false,
                    enforce_order: true,
                    allowed_rows: vec![0, 1],
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

    /// v1のSolveOptionsに変換
    pub fn to_solve_options_v1(&self) -> crate::optimize::SolveOptions {
        crate::optimize::SolveOptions {
            include_fkeys: self.v1.include_fkeys,
            a_ms: self.v1.a_ms,
            b_ms: self.v1.b_ms,
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

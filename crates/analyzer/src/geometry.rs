pub mod build;
pub mod builders;
pub mod fitts;
pub mod policy;
pub mod precompute;
pub mod types;
pub mod visualization;
pub mod zoning;

pub use policy::{ArrowBand, Policy};
pub use precompute::Precompute;
pub use types::{
    Cell, CellId, Finger, Geometry, GeometryConfig, GeometryName, KeyCandidates, RowSpec,
};
pub use visualization::{DebugRenderOptions, LegendPos, RenderMode, render_svg_debug};
pub use zoning::{PinkyEdgeRule, ZonePolicy, apply_zone_policy};

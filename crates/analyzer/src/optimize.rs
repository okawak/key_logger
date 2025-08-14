use std::collections::{BTreeSet, HashMap};

use good_lp::{Expression, ProblemVariables, Solution, SolverModel, Variable, coin_cbc, variable};

use crate::constants::U2MM;
use crate::csv_reader::KeyFreq;
use crate::error::KbOptError;
use crate::geometry::{
    Geometry,
    fitts::euclid_u,
    precompute::Precompute,
    types::{CellId, KeyPlacement, PlacementType},
};
use crate::keys::{KeyId, ParseOptions, all_movable_keys};

pub mod v1;

use v1::{
    ARROW_KEYS, build_adjacency_from_precompute, build_blocks_from_precompute,
    build_candidates_from_precompute, generate_v1_arrow_region, generate_v1_key_candidates,
    is_arrow,
};

/// ソルバ設定・Fitts 係数など
#[derive(Debug, Clone)]
pub struct SolveOptions {
    pub include_fkeys: bool, // F1..F12 を動かすか
    pub a_ms: f64,           // Fitts: a
    pub b_ms: f64,           // Fitts: b
}

impl Default for SolveOptions {
    fn default() -> Self {
        Self {
            include_fkeys: false,
            a_ms: 0.0,
            b_ms: 1.0,
        }
    }
}

/// 解の保持
#[derive(Debug, Clone)]
pub struct SolutionLayout {
    pub objective_ms: f64,
}

pub fn solve_layout(
    geom: &mut Geometry,
    freqs: &KeyFreq,
    opt: &SolveOptions,
) -> Result<SolutionLayout, KbOptError> {
    // 1) 集合を作る - CSVにないキーも含める（count 0として扱う）
    let parse_opt = ParseOptions {
        include_fkeys: opt.include_fkeys,
        ..Default::default()
    };

    let movable: BTreeSet<KeyId> = all_movable_keys(&parse_opt)
        .into_iter()
        .filter(|k| !is_arrow(k))
        .collect();

    // 2) v1: 全キーが全行に配置可能、全空きセルが矢印キー候補
    let movable_vec: Vec<KeyId> = movable.iter().cloned().collect();
    let key_cands = generate_v1_key_candidates(geom, &movable_vec);
    let (arrow_cells, arrow_edges) = generate_v1_arrow_region(geom);

    let precompute = Precompute {
        key_cands,
        arrow_cells,
        arrow_edges,
    };

    // 3) Precomputeから通常キーの候補を変換
    let cands = build_candidates_from_precompute(geom, &movable, &precompute, opt);

    // 4) Precomputeから矢印用ブロックを変換
    let (blocks, _block_index) = build_blocks_from_precompute(geom, &precompute);
    let adj_edges = build_adjacency_from_precompute(&blocks, &precompute);

    // 4) モデルを立てる
    let mut vars = ProblemVariables::new();

    // x^g_{k,j,w}（二値）：通常キー配置変数
    let x_vars: Vec<Variable> = (0..cands.len())
        .map(|_| vars.add(variable().binary()))
        .collect();

    // a^g_u（二値）：ブロック占有変数
    let a_vars: Vec<Variable> = (0..blocks.len())
        .map(|_| vars.add(variable().binary()))
        .collect();

    // m^g_{a,u}：矢印キー配置変数（4種×ブロック）
    let mut m_vars: HashMap<(KeyId, usize), Variable> = HashMap::new();
    for &arrow_key in &ARROW_KEYS {
        for u in 0..blocks.len() {
            m_vars.insert((arrow_key, u), vars.add(variable().binary()));
        }
    }

    // r^g_u（二値）：フロー根変数
    let r_vars: Vec<Variable> = (0..blocks.len())
        .map(|_| vars.add(variable().binary()))
        .collect();

    // f^g_{(u→v)}（連続）：フロー変数
    #[derive(Clone)]
    struct Edge {
        from: usize,
        to: usize,
    }
    let edges: Vec<Edge> = adj_edges
        .iter()
        .map(|(u, v)| Edge { from: *u, to: *v })
        .collect();
    let f_vars: Vec<Variable> = (0..edges.len())
        .map(|_| vars.add(variable().min(0.0)))
        .collect();

    // 正規化された確率値を取得
    let probabilities = freqs.probabilities();
    println!("Probabilities: {:?}", probabilities);

    // 目的関数: Σ p_k·T^g(j,w)·x^g_{k,j,w} + Σ p_a·T^g_arrow(u)·m^g_{a,u}
    let mut obj = Expression::from(0.0);
    // 通常キー項: Σ_{k∈K} Σ_{j∈I^g_k} Σ_{w∈W_k} p_k·T^g(j,w)·x^g_{k,j,w}
    for (i, cand) in cands.iter().enumerate() {
        let p_k = probabilities.get(&cand.key).copied().unwrap_or(0.0);
        if p_k > 0.0 {
            obj += p_k * cand.cost_ms * x_vars[i];
        }
    }
    // 矢印キー項: Σ_{a∈A} Σ_{u∈U^g} p_a·T^g_arrow(u)·m^g_{a,u}
    for (u, blk) in blocks.iter().enumerate() {
        let center_cell = blk.cover_cells[2]; // 中央近傍
        let finger = geom.cells[center_cell.row][center_cell.col].finger;
        let home = geom
            .homes
            .get(&finger)
            .cloned()
            .unwrap_or((blk.center.0, blk.center.1));
        let d_mm = euclid_u(blk.center, home) as f64 * U2MM;
        let w_mm = 1.0f64 * U2MM;
        let t_ms = opt.a_ms + opt.b_ms * ((d_mm / w_mm + 1.0).log2());
        for &arrow_key in &ARROW_KEYS {
            let p_a = probabilities.get(&arrow_key).copied().unwrap_or(0.0);
            if p_a > 0.0 {
                let m_au = m_vars.get(&(arrow_key, u)).unwrap();
                obj += (p_a * t_ms) * *m_au;
            }
        }
    }

    // 目的関数を後で評価するために保存
    let objective_expr = obj.clone();

    // 5) 制約条件

    let mut model = vars.minimise(obj).using(coin_cbc);

    // (i) 一意性制約: Σ_{j∈I^g_k} Σ_{w∈W_k} x^g_{k,j,w} = 1 ∀k∈K
    // movable集合に含まれている全キーは必須配置（頻度0でも配置する）
    for &key in movable.iter() {
        let idxs: Vec<usize> = cands
            .iter()
            .enumerate()
            .filter(|(_, c)| c.key == key)
            .map(|(i, _)| i)
            .collect();
        if !idxs.is_empty() {
            let sum: Expression = idxs.iter().map(|i| x_vars[*i]).sum();
            // movable集合のキーは全て必須配置
            model = model.with(sum.clone().eq(1));
        }
    }

    // (ii) セル非重複制約: Σ C(j',j,w)·x^g_{k,j,w} + Σ B(j',u)·a^g_u + F^g_{j'} ≤ 1 ∀j'∈J_g
    let mut cell_cover_x: HashMap<CellId, Vec<usize>> = HashMap::new();
    for (i, cand) in cands.iter().enumerate() {
        for cid in &cand.cover_cells {
            cell_cover_x.entry(*cid).or_default().push(i);
        }
    }
    let mut cell_cover_a: HashMap<CellId, Vec<usize>> = HashMap::new();
    for (u, blk) in blocks.iter().enumerate() {
        for cid in &blk.cover_cells {
            cell_cover_a.entry(*cid).or_default().push(u);
        }
    }
    for r in 0..geom.cells.len() {
        for c in 0..geom.cells[r].len() {
            let cid = CellId::new(r, c);
            let fixed = if geom.cells[r][c].occupied { 1.0 } else { 0.0 };
            let xs = cell_cover_x.get(&cid).cloned().unwrap_or_default();
            let as_ = cell_cover_a.get(&cid).cloned().unwrap_or_default();
            let mut sum = Expression::from(fixed);
            for i in xs {
                sum += x_vars[i];
            }
            for u in as_ {
                sum += a_vars[u];
            }
            model = model.with(sum << 1);
        }
    }

    // (iii) 矢印キー一意性制約: Σ_{u∈U_g} m^g_{a,u} = 1 ∀a∈A
    for &arrow_key in &ARROW_KEYS {
        let sum: Expression = (0..blocks.len())
            .map(|u| *m_vars.get(&(arrow_key, u)).unwrap())
            .sum();
        model = model.with(sum.eq(1));
    }
    // (iv) ブロック占有整合性制約: Σ_{a∈A} m^g_{a,u} ≤ a^g_u ∀u∈U^g
    for (u, _) in a_vars.iter().enumerate().take(blocks.len()) {
        let sum_a: Expression = ARROW_KEYS
            .iter()
            .map(|&arrow_key| *m_vars.get(&(arrow_key, u)).unwrap())
            .sum();
        model = model.with(sum_a << a_vars[u]);
    }
    // (v) 矢印キー総数制約: Σ_{u∈U_g} a^g_u = 4
    let sum_a_total: Expression = (0..blocks.len()).map(|u| a_vars[u]).sum();
    model = model.with(sum_a_total.eq(4));

    // (vi) 矢印キー連結制約（フロー保存）
    // フロー根一意性: Σ_{u∈U_g} r^g_u = 1
    let sum_r: Expression = (0..blocks.len()).map(|u| r_vars[u]).sum();
    model = model.with(sum_r.eq(1));

    // 出入辺リスト
    let mut in_edges: Vec<Vec<usize>> = vec![Vec::new(); blocks.len()];
    let mut out_edges: Vec<Vec<usize>> = vec![Vec::new(); blocks.len()];
    for (e_idx, e) in edges.iter().enumerate() {
        out_edges[e.from].push(e_idx);
        in_edges[e.to].push(e_idx);
    }
    // フロー保存則: Σ f^g_{(v→u)} - Σ f^g_{(u→v)} = a^g_u - 4r^g_u ∀u∈U_g
    for u in 0..blocks.len() {
        let sum_in: Expression = in_edges[u].iter().map(|&ei| f_vars[ei]).sum();
        let sum_out: Expression = out_edges[u].iter().map(|&ei| f_vars[ei]).sum();
        model = model.with((sum_in - sum_out).eq(a_vars[u] - 4 * r_vars[u]));
    }
    // フロー容量制約: 0 ≤ f^g_{(u→v)} ≤ 3a^g_u ∀(u→v)∈E_g
    for (e_idx, e) in edges.iter().enumerate() {
        model = model.with(f_vars[e_idx] << (3.0 * a_vars[e.from]));
    }

    // 6) 求解
    let sol = model
        .solve()
        .map_err(|e| KbOptError::Other(e.to_string()))?;

    // 7) 解の復元 - 解の情報を直接Geometryに適用
    let objective_ms = sol.eval(&objective_expr);

    // 既存の最適化キーをクリア（固定キーは残す）
    geom.key_placements
        .retain(|_, p| p.placement_type == PlacementType::Fixed);

    // 通常キーの配置を追加
    for (i, cand) in cands.iter().enumerate() {
        if sol.value(x_vars[i]) > 0.5 {
            let (x, y) = crate::constants::cell_to_key_center(
                cand.row * crate::constants::U2CELL,
                cand.start_col,
                cand.w_u,
            );
            geom.key_placements.insert(
                cand.key.to_string(),
                KeyPlacement {
                    placement_type: PlacementType::Optimized,
                    key_id: Some(cand.key),
                    x,
                    y,
                    width_u: cand.w_u,
                    block_id: None,
                },
            );
        }
    }

    // 矢印キーの配置を追加
    for &arrow_key in &ARROW_KEYS {
        for (u, block) in blocks.iter().enumerate() {
            if sol.value(*m_vars.get(&(arrow_key, u)).unwrap()) > 0.5 {
                let start_col = block.id.col_u * 4; // 1u = 4 cells
                let (x, y) = crate::constants::cell_to_key_center(
                    block.id.row_u * crate::constants::U2CELL,
                    start_col,
                    1.0,
                );
                geom.key_placements.insert(
                    arrow_key.to_string(),
                    KeyPlacement {
                        placement_type: PlacementType::Arrow,
                        key_id: Some(arrow_key),
                        x,
                        y,
                        width_u: 1.0,
                        block_id: Some(block.id),
                    },
                );
            }
        }
    }

    Ok(SolutionLayout { objective_ms })
}

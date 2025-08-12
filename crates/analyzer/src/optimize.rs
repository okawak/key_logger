use std::collections::{BTreeSet, HashMap};

use good_lp::{Expression, ProblemVariables, Solution, SolverModel, Variable, coin_cbc, variable};

use crate::error::KbOptError;
use crate::geometry::Geometry;
use crate::geometry::types::*;

/// ソルバ設定・Fitts 係数など
#[derive(Debug, Clone)]
pub struct SolveOptions {
    pub include_function_keys: bool, // F1..F12 を動かすか
    pub a_ms: f64,                   // Fitts: a
    pub b_ms: f64,                   // Fitts: b
    pub u2mm: f64,                   // 1u→mm（例 19.0）
    pub lambda_width: f64,           // サイズペナ（任意。0で無効）
}

impl Default for SolveOptions {
    fn default() -> Self {
        Self {
            include_function_keys: false,
            a_ms: 0.0,
            b_ms: 1.0,
            u2mm: 19.0,
            lambda_width: 0.0,
        }
    }
}

/// 頻度テーブル（キー名→頻度）
pub type KeyFreqs = HashMap<String, u64>;

/// 可動キー集合（矢印は別ハンドリング）
fn is_arrow(name: &str) -> bool {
    matches!(name, "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight")
}
fn is_digit_or_f(name: &str) -> bool {
    name.len() == 1 && name.chars().all(|c| c.is_ascii_digit())
        || (name.starts_with('F') && name[1..].chars().all(|c| c.is_ascii_digit()))
}

/// 幅候補（0.25u 刻み）。数字/F/矢印は 1u 固定。
fn width_candidates_for_key(name: &str) -> Vec<f32> {
    if is_arrow(name) || is_digit_or_f(name) {
        vec![1.0]
    } else {
        // 0.25u 刻みで 0.75..2.00 あたり（お好みで）
        let mut v = Vec::new();
        let mut w = 0.75f32;
        while w <= 2.00 + 1e-6 {
            v.push((w * 100.0).round() / 100.0);
            w += 0.25;
        }
        v
    }
}

/// 配置候補（通常キー）
#[derive(Debug, Clone)]
struct Cand {
    key: String,
    row: usize,
    start_col: usize, // 0.25u index
    w_u: f32,
    cost_ms: f64, // f_k を掛ける前の素コスト
    cover_cells: Vec<CellId>,
}

/// 1u ブロック（矢印用の占有単位）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockId {
    pub row: usize,
    pub bcol: usize, // 1u ブロック列（0.25u 4セルごと）
}
#[derive(Debug, Clone)]
struct Block {
    id: BlockId,
    center: (f32, f32),       // [u]
    cover_cells: [CellId; 4], // この1uが覆う 0.25u セル
}

/// 解の保持
#[derive(Debug, Clone)]
pub struct SolutionLayout {
    pub objective_ms: f64,
    // key -> (row, start_col(0.25u), w_u)
    pub key_place: HashMap<String, (usize, usize, f32)>,
    // arrows: name -> BlockId
    pub arrow_place: HashMap<String, BlockId>,
}

pub fn solve_layout(
    geom: &Geometry,
    freqs: &KeyFreqs,
    opt: &SolveOptions,
) -> Result<SolutionLayout, KbOptError> {
    // 1) 集合を作る
    let mut movable: BTreeSet<String> = freqs.keys().filter(|k| !is_arrow(k)).cloned().collect();

    // Fキーの有無
    if !opt.include_function_keys {
        movable.retain(|k| !(k.starts_with('F') && k[1..].chars().all(|c| c.is_ascii_digit())));
    }

    // 2) 通常キーの候補を生成
    let cands = build_candidates(geom, &movable, opt);

    // 3) 矢印用 1u ブロックと隣接グラフ
    let (blocks, _block_index) = build_blocks_1u(geom);
    let adj_edges = build_block_adjacency(&blocks, geom);

    // 4) モデルを立てる
    let mut vars = ProblemVariables::new();

    // x_i（二値）：通常キー候補
    let x_vars: Vec<Variable> = (0..cands.len())
        .map(|_| vars.add(variable().binary()))
        .collect();

    // a_u（二値）：ブロック使用
    let a_vars: Vec<Variable> = (0..blocks.len())
        .map(|_| vars.add(variable().binary()))
        .collect();

    // m_{d,u}：矢印4種×ブロック
    let arrow_names = ["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"];
    let mut m_vars: HashMap<(&'static str, usize), Variable> = HashMap::new();
    for d in &arrow_names {
        for u in 0..blocks.len() {
            m_vars.insert((*d, u), vars.add(variable().binary()));
        }
    }

    // r_u（二値）：フローの根
    let r_vars: Vec<Variable> = (0..blocks.len())
        .map(|_| vars.add(variable().binary()))
        .collect();

    // f_e（連結フロー）
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

    // 目的関数
    let mut obj = Expression::from(0.0);
    for (i, cand) in cands.iter().enumerate() {
        let fk = *freqs.get(&cand.key).unwrap_or(&0u64) as f64;
        let width_pen = opt.lambda_width * (cand.w_u as f64);
        obj += (fk * (cand.cost_ms + width_pen)) * x_vars[i];
    }
    // 矢印（各ブロックに置いたときの Fitts コスト）
    for (u, blk) in blocks.iter().enumerate() {
        let center_cell = blk.cover_cells[2]; // 中央近傍
        let finger = geom.cells[center_cell.row][center_cell.col].finger;
        let home = geom
            .homes
            .get(&finger)
            .cloned()
            .unwrap_or((blk.center.0, blk.center.1));
        let d_u = euclid_u(blk.center, home) as f64 * opt.u2mm;
        let w_mm = 1.0f64 * opt.u2mm;
        let t_ms = opt.a_ms + opt.b_ms * ((d_u / w_mm + 1.0).log2());
        for &d in &arrow_names {
            let fd = *freqs.get(d).unwrap_or(&0u64) as f64;
            if fd > 0.0 {
                let mv = m_vars.get(&(d, u)).unwrap();
                obj += (fd * t_ms) * *mv;
            }
        }
    }

    // 目的関数を後で評価するために保存
    let objective_expr = obj.clone();

    // 5) 制約

    let mut model = vars.minimise(obj).using(coin_cbc);

    // (i) 各キーは高々1配置（=必須なら Exactly1 にする）
    for key in movable.iter() {
        let idxs: Vec<usize> = cands
            .iter()
            .enumerate()
            .filter(|(_, c)| &c.key == key)
            .map(|(i, _)| i)
            .collect();
        if !idxs.is_empty() {
            let sum: Expression = idxs.iter().map(|i| x_vars[*i]).sum();
            // 必須配置（頻度>0なら）
            if *freqs.get(key).unwrap_or(&0) > 0 {
                model = model.with(sum.clone().eq(1));
            } else {
                model = model.with(sum.clone() << 1);
            }
        }
    }

    // (ii) セル非重複（固定文字 + 通常キー + 矢印ブロック）
    // 各セル j'： Σ x(セルを覆う) + Σ a(ブロックがそのセルを覆う) + fixed <= 1
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
    for r in 0..geom.cfg.rows.len() {
        for c in 0..geom.cells_per_row {
            let cid = CellId::new(r, c);
            let fixed = if geom.cells[r][c].fixed_occupied {
                1.0
            } else {
                0.0
            };
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

    // (iii) 矢印：各方向は1つのブロックに割り当て
    for &d in &arrow_names {
        let sum: Expression = (0..blocks.len())
            .map(|u| *m_vars.get(&(d, u)).unwrap())
            .sum();
        // 頻度0なら置かない/置いても重みゼロだが、通常はExactly1を維持
        model = model.with(sum.eq(1));
    }
    // (iv) ブロック使用 a_u と m_{d,u} の整合
    for (u, _) in a_vars.iter().enumerate().take(blocks.len()) {
        let sum_d: Expression = arrow_names
            .iter()
            .map(|d| *m_vars.get(&(*d, u)).unwrap())
            .sum();
        model = model.with(sum_d << a_vars[u]);
    }
    // (v) 使用ブロックはちょうど4
    let sum_a: Expression = (0..blocks.len()).map(|u| a_vars[u]).sum();
    model = model.with(sum_a.eq(4));

    // (vi) 連結性（単一フロー）
    //   Σin f - Σout f = a_u - 4 r_u
    //   Σ r_u = 1
    //   0 <= f_e <= 3 a_from
    let sum_r: Expression = (0..blocks.len()).map(|u| r_vars[u]).sum();
    model = model.with(sum_r.eq(1));

    // 出入辺リスト
    let mut in_edges: Vec<Vec<usize>> = vec![Vec::new(); blocks.len()];
    let mut out_edges: Vec<Vec<usize>> = vec![Vec::new(); blocks.len()];
    for (e_idx, e) in edges.iter().enumerate() {
        out_edges[e.from].push(e_idx);
        in_edges[e.to].push(e_idx);
    }
    for u in 0..blocks.len() {
        let sum_in: Expression = in_edges[u].iter().map(|&ei| f_vars[ei]).sum();
        let sum_out: Expression = out_edges[u].iter().map(|&ei| f_vars[ei]).sum();
        model = model.with((sum_in - sum_out).eq(a_vars[u] - 4 * r_vars[u]));
    }
    for (e_idx, e) in edges.iter().enumerate() {
        model = model.with(f_vars[e_idx] << (3.0 * a_vars[e.from]));
    }

    // 6) 求解
    let sol = model
        .solve()
        .map_err(|e| KbOptError::Other(e.to_string()))?;

    // 7) 解の復元
    let mut key_place = HashMap::new();
    for (i, cand) in cands.iter().enumerate() {
        if sol.value(x_vars[i]) > 0.5 {
            key_place.insert(cand.key.clone(), (cand.row, cand.start_col, cand.w_u));
        }
    }
    let mut arrow_place = HashMap::new();
    for &d in &arrow_names {
        for (u, block) in blocks.iter().enumerate() {
            if sol.value(*m_vars.get(&(d, u)).unwrap()) > 0.5 {
                arrow_place.insert(d.to_string(), block.id);
            }
        }
    }
    let objective_ms = sol.eval(&objective_expr);

    Ok(SolutionLayout {
        objective_ms,
        key_place,
        arrow_place,
    })
}

/* ----------------- 内部：候補生成・ブロック構築 ----------------- */

fn build_candidates(geom: &Geometry, movable: &BTreeSet<String>, opt: &SolveOptions) -> Vec<Cand> {
    let mut out = Vec::new();
    for key in movable {
        let widths = width_candidates_for_key(key);
        let fk_dummy = 1.0f64; // ここでは素コスト（頻度は後で掛ける）
        for r in 0..geom.cfg.rows.len() {
            let row = &geom.cfg.rows[r];
            for &w_u in &widths {
                let w_cells = cells_from_u(w_u);
                if w_cells == 0 {
                    continue;
                }
                for c0 in 0..=geom.cells_per_row - w_cells {
                    // 固定占有に当たらないかチェック
                    if (c0..c0 + w_cells).any(|c| geom.cells[r][c].fixed_occupied) {
                        continue;
                    }
                    // 中心セルの指でホームを取る
                    let c_center = c0 + w_cells / 2;
                    let (cx, cy) = {
                        let x0 = row.offset_u + c0 as f32 * CELL_U;
                        let x1 = row.offset_u + (c0 + w_cells) as f32 * CELL_U;
                        ((x0 + x1) * 0.5, row.base_y_u)
                    };
                    let finger = geom.cells[r][c_center].finger;
                    let home = geom.homes.get(&finger).cloned().unwrap_or((cx, cy));
                    let d_mm = (euclid_u((cx, cy), home) as f64) * opt.u2mm;
                    let w_mm = (w_u as f64) * opt.u2mm;
                    let t_ms = opt.a_ms + opt.b_ms * ((d_mm / w_mm + 1.0).log2());
                    let cover_cells: Vec<CellId> =
                        (c0..c0 + w_cells).map(|cc| CellId::new(r, cc)).collect();
                    out.push(Cand {
                        key: key.clone(),
                        row: r,
                        start_col: c0,
                        w_u,
                        cost_ms: t_ms * fk_dummy,
                        cover_cells,
                    });
                }
            }
        }
    }
    out
}

fn euclid_u(a: (f32, f32), b: (f32, f32)) -> f32 {
    let dx = a.0 - b.0;
    let dy = a.1 - b.1;
    (dx * dx + dy * dy).sqrt()
}

fn build_blocks_1u(geom: &Geometry) -> (Vec<Block>, HashMap<BlockId, usize>) {
    let mut blocks = Vec::new();
    let mut index = HashMap::new();
    for r in 0..geom.cfg.rows.len() {
        let row = &geom.cfg.rows[r];
        let bcols = geom.cells_per_row / cells_from_u(ONE_U);
        for b in 0..bcols {
            let start_col = b * cells_from_u(ONE_U);
            // 固定占有に当たらないか
            if (start_col..start_col + cells_from_u(ONE_U)).any(|c| geom.cells[r][c].fixed_occupied)
            {
                continue;
            }
            let x0 = row.offset_u + start_col as f32 * CELL_U;
            let cx = x0 + 0.5 * ONE_U;
            let cy = row.base_y_u;
            let ids = [
                CellId::new(r, start_col),
                CellId::new(r, start_col + 1),
                CellId::new(r, start_col + 2),
                CellId::new(r, start_col + 3),
            ];
            let id = BlockId { row: r, bcol: b };
            let idx = blocks.len();
            blocks.push(Block {
                id,
                center: (cx, cy),
                cover_cells: ids,
            });
            index.insert(id, idx);
        }
    }
    (blocks, index)
}

/// 1u ブロックの隣接（8近傍：水平/垂直/斜め いずれかが接する）
fn build_block_adjacency(blocks: &[Block], geom: &Geometry) -> Vec<(usize, usize)> {
    let mut edges = Vec::new();
    for (i, bi) in blocks.iter().enumerate() {
        for (j, bj) in blocks.iter().enumerate().skip(i + 1) {
            if blocks_touch(bi, bj, geom) {
                edges.push((i, j));
                edges.push((j, i));
            }
        }
    }
    edges
}
fn blocks_touch(a: &Block, b: &Block, geom: &Geometry) -> bool {
    // a,b の 1u 矩形が辺または角で接していれば true
    let rect = |blk: &Block| {
        // 各セルは 0.25u 正方形。ブロックは横1u, 縦1u
        let r = blk.cover_cells[0].row;
        let c0 = blk.cover_cells[0].col;
        let x0 = geom.cfg.rows[r].offset_u + c0 as f32 * CELL_U;
        let y0 = geom.cfg.rows[r].base_y_u - 0.5;
        (x0, y0, x0 + ONE_U, y0 + 1.0)
    };
    let (ax0, ay0, ax1, ay1) = rect(a);
    let (bx0, by0, bx1, by1) = rect(b);
    let sep_x = (ax1 < bx0) || (bx1 < ax0);
    let sep_y = (ay1 < by0) || (by1 < ay0);
    if sep_x || sep_y {
        // 角接触（端点一致）も許す：距離ゼロなら true
        let dx = if ax1 < bx0 {
            bx0 - ax1
        } else if bx1 < ax0 {
            ax0 - bx1
        } else {
            0.0
        };
        let dy = if ay1 < by0 {
            by0 - ay1
        } else if by1 < ay0 {
            ay0 - by1
        } else {
            0.0
        };
        (dx.abs() <= 1e-6) && (dy.abs() <= 1e-6)
    } else {
        true // 矩形が重なる or 辺接触
    }
}

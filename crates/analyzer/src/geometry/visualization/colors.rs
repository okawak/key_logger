use super::super::types::Finger;
use crate::optimize::KeyFreqs;

/// 指の色を返す
pub fn color_of(fgr: Finger) -> &'static str {
    use Finger::*;
    match fgr {
        LPinky => "#ff9aa2",  // 薄いピンク
        LRing => "#ffbfa3",   // 薄いオレンジ
        LMiddle => "#fff4a3", // 薄い黄色
        LIndex => "#b9ffb7",  // 薄い緑
        LThumb => "#b5d6ff",  // 薄い青
        RThumb => "#98c7ff",  // 薄い青 (少し濃い)
        RIndex => "#a7fff0",  // 薄いシアン
        RMiddle => "#fff08a", // 薄い黄色 (少し濃い)
        RRing => "#ffc89b",   // 薄いオレンジ (少し濃い)
        RPinky => "#ff8c94",  // 薄いピンク (少し濃い)
    }
}

/// 指のラベル文字を返す
pub fn finger_label(fgr: Finger) -> &'static str {
    use Finger::*;
    match fgr {
        LPinky => "LP",
        LRing => "LR",
        LMiddle => "LM",
        LIndex => "LI",
        LThumb => "LT",
        RThumb => "RT",
        RIndex => "RI",
        RMiddle => "RM",
        RRing => "RR",
        RPinky => "RP",
    }
}

/// キーの色を頻度に基づいて決定
pub fn get_key_color(key_name: &str, freqs: &KeyFreqs) -> &'static str {
    let freq = *freqs.get(key_name).unwrap_or(&0);

    if freq >= 1000 {
        "#ff6b6b" // 赤 (高頻度)
    } else if freq >= 500 {
        "#feca57" // 黄 (中高頻度)
    } else if freq >= 100 {
        "#48dbfb" // 青 (中頻度)
    } else {
        "#ddd" // 灰 (低頻度)
    }
}

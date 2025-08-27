use crate::{
    config::Config,
    constants::{DEFAULT_FKEYS_MAX, MAX_DIGIT},
};

use std::fmt;

/// letter keys
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LetterKey {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
}

/// Symbol keys
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SymbolKey {
    Backtick,  // `
    Minus,     // -
    Equal,     // =
    LBracket,  // [
    RBracket,  // ]
    Backslash, // \
    Semicolon, // ;
    Quote,     // '
    Comma,     // ,
    Period,    // .
    Slash,     // /
}

/// Arrow keys
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArrowKey {
    Left,
    Down,
    Up,
    Right,
}

/// Modifier keys for layer switching
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ModifierKey {
    Layer1,
    Layer2,
    Layer3,
}

/// Optimized key identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum KeyId {
    // letter
    Letter(LetterKey),
    // digit
    Digit(u8), // 0..9
    // symbols (US)
    Symbol(SymbolKey),
    Tab,
    Escape,
    CapsLock,
    Delete,
    Backspace,
    Space,
    Enter,
    ShiftL,
    ShiftR,
    CtrlL,
    CtrlR,
    AltL,
    AltR,
    MetaL,
    MetaR,
    Function(u8),
    Arrow(ArrowKey),
    Modifier(ModifierKey),
    // navigation keys
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    // numeric keypad keys
    NumpadDigit(u8), // 0..9
    NumpadAdd,
    NumpadSubtract,
    NumpadMultiply,
    NumpadDivide,
    NumpadEnter,
    NumpadEquals,
    NumpadDecimal,
}

impl fmt::Display for KeyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use KeyId::*;
        match self {
            Letter(l) => write!(f, "{:?}", l),
            Digit(d) => write!(f, "{}", d),
            Symbol(s) => write!(f, "{:?}", s),
            Tab => write!(f, "Tab"),
            Escape => write!(f, "Escape"),
            CapsLock => write!(f, "CapsLock"),
            Delete => write!(f, "Delete"),
            Backspace => write!(f, "Backspace"),
            Space => write!(f, "Space"),
            Enter => write!(f, "Enter"),
            ShiftL => write!(f, "LeftShift"),
            ShiftR => write!(f, "RightShift"),
            CtrlL => write!(f, "LeftControl"),
            CtrlR => write!(f, "RightControl"),
            AltL => write!(f, "LeftAlt"),
            AltR => write!(f, "RightAlt"),
            MetaL => write!(f, "LeftMeta"),
            MetaR => write!(f, "RightMeta"),
            Function(n) => write!(f, "F{}", n),
            Arrow(a) => write!(f, "Arrow{:?}", a),
            Modifier(m) => write!(f, "Modifier{:?}", m),
            Home => write!(f, "Home"),
            End => write!(f, "End"),
            PageUp => write!(f, "PageUp"),
            PageDown => write!(f, "PageDown"),
            Insert => write!(f, "Insert"),
            NumpadDigit(d) => write!(f, "Numpad{}", d),
            NumpadAdd => write!(f, "NumpadAdd"),
            NumpadSubtract => write!(f, "NumpadSubtract"),
            NumpadMultiply => write!(f, "NumpadMultiply"),
            NumpadDivide => write!(f, "NumpadDivide"),
            NumpadEnter => write!(f, "NumpadEnter"),
            NumpadEquals => write!(f, "NumpadEquals"),
            NumpadDecimal => write!(f, "NumpadDecimal"),
        }
    }
}

pub fn str_to_keyid(str: &str) -> Option<KeyId> {
    use LetterKey::*;
    match str {
        // letters
        "A" => Some(KeyId::Letter(A)),
        "B" => Some(KeyId::Letter(B)),
        "C" => Some(KeyId::Letter(C)),
        "D" => Some(KeyId::Letter(D)),
        "E" => Some(KeyId::Letter(E)),
        "F" => Some(KeyId::Letter(F)),
        "G" => Some(KeyId::Letter(G)),
        "H" => Some(KeyId::Letter(H)),
        "I" => Some(KeyId::Letter(I)),
        "J" => Some(KeyId::Letter(J)),
        "K" => Some(KeyId::Letter(K)),
        "L" => Some(KeyId::Letter(L)),
        "M" => Some(KeyId::Letter(M)),
        "N" => Some(KeyId::Letter(N)),
        "O" => Some(KeyId::Letter(O)),
        "P" => Some(KeyId::Letter(P)),
        "Q" => Some(KeyId::Letter(Q)),
        "R" => Some(KeyId::Letter(R)),
        "S" => Some(KeyId::Letter(S)),
        "T" => Some(KeyId::Letter(T)),
        "U" => Some(KeyId::Letter(U)),
        "V" => Some(KeyId::Letter(V)),
        "W" => Some(KeyId::Letter(W)),
        "X" => Some(KeyId::Letter(X)),
        "Y" => Some(KeyId::Letter(Y)),
        "Z" => Some(KeyId::Letter(Z)),
        // digits
        "0" => Some(KeyId::Digit(0)),
        "1" => Some(KeyId::Digit(1)),
        "2" => Some(KeyId::Digit(2)),
        "3" => Some(KeyId::Digit(3)),
        "4" => Some(KeyId::Digit(4)),
        "5" => Some(KeyId::Digit(5)),
        "6" => Some(KeyId::Digit(6)),
        "7" => Some(KeyId::Digit(7)),
        "8" => Some(KeyId::Digit(8)),
        "9" => Some(KeyId::Digit(9)),
        _ => None, // まだ必要性がないのでここまで
    }
}

pub fn parse_key_label(label: &str) -> Option<KeyId> {
    use ArrowKey::*;
    use KeyId::*;
    //use ModifierKey::*;
    use SymbolKey::*;

    let s = label.trim();
    if s.is_empty() {
        return None;
    }

    // One character keys
    // - ASCII alphabetic characters are not considered optimized keys
    // - ASCII digits are considered keys (0-9)
    if s.len() == 1 {
        let ch = s.as_bytes()[0];
        if ch.is_ascii_alphabetic() {
            return None;
        }
        if ch.is_ascii_digit() {
            return Some(Digit(ch - b'0'));
        }
    }

    let t = s.to_ascii_lowercase();
    let t = t.as_str();

    if let Some(rest) = t.strip_prefix("key")
        && let Ok(n) = rest.parse::<u8>()
        && n <= MAX_DIGIT
    {
        return Some(Digit(n));
    }

    // Symbol keys (US layout)
    match t {
        "grave" | "`" => return Some(Symbol(Backtick)),
        "minus" | "-" => return Some(Symbol(Minus)),
        "equal" | "=" => return Some(Symbol(Equal)),
        "leftbracket" | "[" => return Some(Symbol(LBracket)),
        "rightbracket" | "]" => return Some(Symbol(RBracket)),
        "backslash" | "\\" => return Some(Symbol(Backslash)),
        "semicolon" | ";" => return Some(Symbol(Semicolon)),
        "apostrophe" | "'" => return Some(Symbol(Quote)),
        "comma" | "," => return Some(Symbol(Comma)),
        "period" | "dot" | "." => return Some(Symbol(Period)),
        "slash" | "/" => return Some(Symbol(Slash)),
        _ => {}
    }

    match t {
        "tab" => return Some(Tab),
        "escape" => return Some(Escape),
        "capslock" => return Some(CapsLock),
        "delete" => return Some(Delete),
        "backspace" => return Some(Backspace),
        "space" | "spacebar" => return Some(Space),
        "enter" | "return" => return Some(Enter),
        "leftshift" => return Some(ShiftL),
        "rightshift" => return Some(ShiftR),
        "leftcontrol" => return Some(CtrlL),
        "rightcontrol" => return Some(CtrlR),
        "leftalt" | "loption" => return Some(AltL),
        "rightalt" | "roption" => return Some(AltR),
        "leftmeta" | "command" => return Some(MetaL),
        "rightmeta" | "rcommand" => return Some(MetaR),
        _ => {}
    }

    match t {
        "arrowleft" | "left" => return Some(Arrow(Left)),
        "arrowright" | "right" => return Some(Arrow(Right)),
        "arrowup" | "up" => return Some(Arrow(Up)),
        "arrowdown" | "down" => return Some(Arrow(Down)),
        _ => {}
    }

    // Modifier keys for layer switching
    //if opt.include_modifiers {
    //    match t {
    //        "layer1" | "modifier1" => return Some(Modifier(Layer1)),
    //        "layer2" | "modifier2" => return Some(Modifier(Layer2)),
    //        "layer3" | "modifier3" => return Some(Modifier(Layer3)),
    //        _ => {}
    //    }
    //}

    //if opt.include_navigation {
    //    match t {
    //        "home" => return Some(Home),
    //        "end" => return Some(End),
    //        "pageup" => return Some(PageUp),
    //        "pagedown" => return Some(PageDown),
    //        "insert" => return Some(Insert),
    //        _ => {}
    //    }
    //}

    //if opt.include_fkeys
    //    && let Some(rest) = t.strip_prefix('f')
    //    && let Ok(n) = rest.parse::<u8>()
    //    && 1 <= n
    //    && n <= opt.fkeys_max
    //{
    //    return Some(Function(n));
    //}

    //if opt.include_numpad
    //    && let Some(rest) = t.strip_prefix("numpad")
    //{
    //    match rest {
    //        "add" => return Some(NumpadAdd),
    //        "subtract" => return Some(NumpadSubtract),
    //        "multiply" => return Some(NumpadMultiply),
    //        "divide" => return Some(NumpadDivide),
    //        "enter" => return Some(NumpadEnter),
    //        "equals" => return Some(NumpadEquals),
    //        "decimal" => return Some(NumpadDecimal),
    //        _ => {
    //            if let Ok(n) = rest.parse::<u8>()
    //                && n <= MAX_NUMPAD_DIGIT
    //            {
    //                return Some(NumpadDigit(n));
    //            }
    //        }
    //    }
    //}

    None
}

pub fn all_movable_keys(config: &Config) -> Vec<KeyId> {
    use KeyId::*;
    use SymbolKey::*;

    let mut v = Vec::new();

    // include_digitがtrueの場合、最適化候補に入れる
    if config.solver.include_digits {
        for d in 0..=9 {
            v.push(Digit(d));
        }
    }

    for s in [
        Backtick, Minus, Equal, LBracket, RBracket, Backslash, Semicolon, Quote, Comma, Period,
        Slash,
    ] {
        v.push(Symbol(s));
    }
    v.extend([
        Tab, Escape, CapsLock, Delete, Backspace, Space, Enter, ShiftL, ShiftR, CtrlL, CtrlR, AltL,
        AltR, MetaL, MetaR,
    ]);

    if config.solver.include_fkeys {
        for n in 1..=DEFAULT_FKEYS_MAX {
            v.push(Function(n));
        }
    }

    v.extend([
        Arrow(ArrowKey::Left),
        Arrow(ArrowKey::Down),
        Arrow(ArrowKey::Up),
        Arrow(ArrowKey::Right),
    ]);

    //if opt.include_modifiers {
    //    v.extend([
    //        Modifier(ModifierKey::Layer1),
    //        Modifier(ModifierKey::Layer2),
    //        Modifier(ModifierKey::Layer3),
    //    ]);
    //}

    //if opt.include_navigation {
    //    v.extend([Home, End, PageUp, PageDown, Insert]);
    //}
    //if opt.include_numpad {
    //    for n in 0..=9 {
    //        v.push(NumpadDigit(n));
    //    }
    //    v.extend([
    //        NumpadAdd,
    //        NumpadSubtract,
    //        NumpadMultiply,
    //        NumpadDivide,
    //        NumpadEnter,
    //        NumpadEquals,
    //        NumpadDecimal,
    //    ]);
    //}
    v
}

pub fn allowed_widths(k: &KeyId) -> &'static [f32] {
    use KeyId::*;
    match k {
        Digit(_) | Function(_) | Arrow(_) | Modifier(_) | NumpadDigit(_) | NumpadAdd
        | NumpadSubtract | NumpadMultiply | NumpadDivide | NumpadEnter | NumpadEquals
        | NumpadDecimal => ONE,
        _ => W_VAR,
    }
}

static ONE: &[f32] = &[1.00];
static W_VAR: &[f32] = &[1.00, 1.25, 1.50, 1.75, 2.00, 2.25, 2.50];

// === 共通クラスタ定義（全バージョン共通） ===

/// 矢印キーのクラスタ定義
/// CLAUDE.md仕様：4個の連結ブロック
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArrowClusterId {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

impl ArrowClusterId {
    /// 矢印キーの標準配列（4個）
    pub fn all() -> [ArrowClusterId; 4] {
        [
            ArrowClusterId::Up,
            ArrowClusterId::Down,
            ArrowClusterId::Left,
            ArrowClusterId::Right,
        ]
    }

    /// ArrowClusterIdからKeyIdに変換
    pub fn to_key_id(self) -> KeyId {
        match self {
            ArrowClusterId::Up => KeyId::Arrow(ArrowKey::Up),
            ArrowClusterId::Down => KeyId::Arrow(ArrowKey::Down),
            ArrowClusterId::Left => KeyId::Arrow(ArrowKey::Left),
            ArrowClusterId::Right => KeyId::Arrow(ArrowKey::Right),
        }
    }

    /// KeyIdからArrowClusterIdに変換
    pub fn from_key_id(key_id: KeyId) -> Option<ArrowClusterId> {
        match key_id {
            KeyId::Arrow(ArrowKey::Up) => Some(ArrowClusterId::Up),
            KeyId::Arrow(ArrowKey::Down) => Some(ArrowClusterId::Down),
            KeyId::Arrow(ArrowKey::Left) => Some(ArrowClusterId::Left),
            KeyId::Arrow(ArrowKey::Right) => Some(ArrowClusterId::Right),
            _ => None,
        }
    }
}

/// 数字キーのクラスタ定義
/// CLAUDE.md仕様：10個の連結ブロック（1,2,3,4,5,6,7,8,9,0の順序）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DigitClusterId {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Zero = 0,
}

impl DigitClusterId {
    /// 数字キーの標準配列（10個、順序固定）
    pub fn sequence() -> [DigitClusterId; 10] {
        [
            DigitClusterId::One,
            DigitClusterId::Two,
            DigitClusterId::Three,
            DigitClusterId::Four,
            DigitClusterId::Five,
            DigitClusterId::Six,
            DigitClusterId::Seven,
            DigitClusterId::Eight,
            DigitClusterId::Nine,
            DigitClusterId::Zero,
        ]
    }

    /// DigitClusterIdからKeyIdに変換
    pub fn to_key_id(self) -> KeyId {
        KeyId::Digit(self as u8)
    }

    /// KeyIdからDigitClusterIdに変換
    pub fn from_key_id(key_id: KeyId) -> Option<DigitClusterId> {
        match key_id {
            KeyId::Digit(1) => Some(DigitClusterId::One),
            KeyId::Digit(2) => Some(DigitClusterId::Two),
            KeyId::Digit(3) => Some(DigitClusterId::Three),
            KeyId::Digit(4) => Some(DigitClusterId::Four),
            KeyId::Digit(5) => Some(DigitClusterId::Five),
            KeyId::Digit(6) => Some(DigitClusterId::Six),
            KeyId::Digit(7) => Some(DigitClusterId::Seven),
            KeyId::Digit(8) => Some(DigitClusterId::Eight),
            KeyId::Digit(9) => Some(DigitClusterId::Nine),
            KeyId::Digit(0) => Some(DigitClusterId::Zero),
            _ => None,
        }
    }

    /// 順序インデックス（0-9）
    pub fn sequence_index(self) -> usize {
        match self {
            DigitClusterId::One => 0,
            DigitClusterId::Two => 1,
            DigitClusterId::Three => 2,
            DigitClusterId::Four => 3,
            DigitClusterId::Five => 4,
            DigitClusterId::Six => 5,
            DigitClusterId::Seven => 6,
            DigitClusterId::Eight => 7,
            DigitClusterId::Nine => 8,
            DigitClusterId::Zero => 9,
        }
    }
}

/// クラスタ設定（矢印・数字共通）
#[derive(Debug, Clone)]
pub struct ClusterConfig {
    /// 矢印クラスタの有効化
    pub enable_arrows: bool,
    /// 数字クラスタの有効化
    pub enable_digits: bool,
    /// 数字順序の強制（1,2,3,...,9,0）
    pub enforce_digit_sequence: bool,
    /// 配置許可行（0-based）
    pub allowed_rows: Vec<usize>,
    /// 水平配置の強制
    pub enforce_horizontal: bool,
    /// 左端揃え - 全ての行の左端位置を一致させる
    pub align_left_edge: bool,
    /// 右端揃え - 全ての行の右端位置を一致させる
    pub align_right_edge: bool,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            enable_arrows: true,
            enable_digits: true,
            enforce_digit_sequence: true,
            allowed_rows: vec![0, 1], // 上部2行
            enforce_horizontal: false,
            align_left_edge: false,
            align_right_edge: false,
        }
    }
}

use std::fmt;

/// Symbol keys
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArrowKey {
    Left,
    Down,
    Up,
    Right,
}

const MAX_DIGIT: u8 = 9;
const MAX_NUMPAD_DIGIT: u8 = 9;

/// Optimized key identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyId {
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

#[derive(Debug, Clone)]
pub struct ParseOptions {
    pub include_fkeys: bool,
    pub fkeys_max: u8,
    pub include_navigation: bool,
    pub include_numpad: bool,
    pub strict_unknown_keys: bool,
}

pub const DEFAULT_FKEYS_MAX: u8 = 12;

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            include_fkeys: false,
            fkeys_max: DEFAULT_FKEYS_MAX,
            include_navigation: false,
            include_numpad: false,
            strict_unknown_keys: false,
        }
    }
}

pub fn parse_key_label(label: &str, opt: &ParseOptions) -> Option<KeyId> {
    use ArrowKey::*;
    use KeyId::*;
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

    if opt.include_navigation {
        match t {
            "home" => return Some(Home),
            "end" => return Some(End),
            "pageup" => return Some(PageUp),
            "pagedown" => return Some(PageDown),
            "insert" => return Some(Insert),
            _ => {}
        }
    }

    if opt.include_fkeys
        && let Some(rest) = t.strip_prefix('f')
        && let Ok(n) = rest.parse::<u8>()
        && 1 <= n
        && n <= opt.fkeys_max
    {
        return Some(Function(n));
    }

    if opt.include_numpad
        && let Some(rest) = t.strip_prefix("numpad")
    {
        match rest {
            "add" => return Some(NumpadAdd),
            "subtract" => return Some(NumpadSubtract),
            "multiply" => return Some(NumpadMultiply),
            "divide" => return Some(NumpadDivide),
            "enter" => return Some(NumpadEnter),
            "equals" => return Some(NumpadEquals),
            "decimal" => return Some(NumpadDecimal),
            _ => {
                if let Ok(n) = rest.parse::<u8>()
                    && n <= MAX_NUMPAD_DIGIT
                {
                    return Some(NumpadDigit(n));
                }
            }
        }
    }

    None
}

pub fn all_movable_keys(opt: &ParseOptions) -> Vec<KeyId> {
    use KeyId::*;
    let mut v = vec![];

    for d in 0..=9 {
        v.push(Digit(d));
    }
    use SymbolKey::*;
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

    if opt.include_fkeys {
        for n in 1..=opt.fkeys_max {
            v.push(Function(n));
        }
    }

    v.extend([
        Arrow(ArrowKey::Left),
        Arrow(ArrowKey::Down),
        Arrow(ArrowKey::Up),
        Arrow(ArrowKey::Right),
    ]);

    if opt.include_navigation {
        v.extend([Home, End, PageUp, PageDown, Insert]);
    }
    if opt.include_numpad {
        for n in 0..=9 {
            v.push(NumpadDigit(n));
        }
        v.extend([
            NumpadAdd,
            NumpadSubtract,
            NumpadMultiply,
            NumpadDivide,
            NumpadEnter,
            NumpadEquals,
            NumpadDecimal,
        ]);
    }
    v
}

pub fn allowed_widths(k: &KeyId) -> &'static [f32] {
    use KeyId::*;
    match k {
        Digit(_) | Function(_) | Arrow(_) | NumpadDigit(_) | NumpadAdd | NumpadSubtract
        | NumpadMultiply | NumpadDivide | NumpadEnter | NumpadEquals | NumpadDecimal => ONE,
        _ => W_VAR,
    }
}

static ONE: &[f32] = &[1.00];
static W_VAR: &[f32] = &[1.00, 1.25, 1.50, 1.75, 2.00, 2.25, 2.50];

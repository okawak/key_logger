// Digitタイプのキーと数字キーの表示対応を修正

// render_all_keys関数内のmatch文を以下のように修正：

// 1. PlacementType::Digitケースを追加：
            PlacementType::Digit => {
                // 数字キーは黄色の塗りつぶし
                renderer.draw_rect(
                    key_left_px,
                    key_top_px,
                    width_px,
                    height_px,
                    Colors::LIGHT_YELLOW,
                );
                renderer.draw_rect_outline(
                    key_left_px,
                    key_top_px,
                    width_px,
                    height_px,
                    Colors::BLACK,
                );
            }

// 2. display_textのmatch文を拡張して数字キー対応を追加：
        let display_text = match key_name.as_str() {
            "ArrowUp" => "↑",
            "ArrowDown" => "↓",
            "ArrowLeft" => "←",
            "ArrowRight" => "→",
            "Backslash" => r"\",
            "Slash" => "/",
            "RBracket" => "]",
            "LBracket" => "[",
            "Semicolon" => ";",
            "Equal" => "=",
            "Minus" => "-",
            "Backtick" => "`",
            "Quote" => "'",
            "RightShift" => "R⇧",
            "Period" => ".",
            "Comma" => ",",
            "LeftShift" => "L⇧",
            "Space" => "△",
            "LeftControl" => "LCtrl",
            "RightControl" => "RCtrl",
            "LeftAlt" => "LAlt",
            "RightAlt" => "RAlt",
            "LeftMeta" => "LMeta",
            "RightMeta" => "RMeta",
            "Backspace" => "BS",
            "Delete" => "Del",
            "CapsLock" => "Caps",
            "Escape" => "Esc",
            "Tab" => "Tab",
            "Enter" => "Enter",
            // 数字キーの対応を追加
            "Digit(0)" => "0",
            "Digit(1)" => "1",
            "Digit(2)" => "2",
            "Digit(3)" => "3",
            "Digit(4)" => "4",
            "Digit(5)" => "5",
            "Digit(6)" => "6",
            "Digit(7)" => "7",
            "Digit(8)" => "8",
            "Digit(9)" => "9",
            // KeyIdのDebug形式に対応
            s if s.starts_with("Digit(") => {
                // "Digit(3)" -> "3"
                s.trim_start_matches("Digit(")
                    .trim_end_matches(")")
            },
            s if s.starts_with("Symbol(") => {
                // "Symbol(Comma)" -> "," など、個別マッピングが必要
                match s {
                    "Symbol(Comma)" => ",",
                    "Symbol(Period)" => ".",
                    "Symbol(Slash)" => "/",
                    "Symbol(Semicolon)" => ";",
                    "Symbol(Quote)" => "'",
                    "Symbol(LBracket)" => "[",
                    "Symbol(RBracket)" => "]",
                    "Symbol(Backslash)" => r"\",
                    "Symbol(Backtick)" => "`",
                    "Symbol(Minus)" => "-",
                    "Symbol(Equal)" => "=",
                    _ => s,
                }
            },
            s if s.starts_with("Arrow(") => {
                // "Arrow(Up)" -> "↑"
                match s {
                    "Arrow(Up)" => "↑",
                    "Arrow(Down)" => "↓",
                    "Arrow(Left)" => "←",
                    "Arrow(Right)" => "→",
                    _ => s,
                }
            },
            _ => key_name.as_str(),
        };
# Key Logger

クロスプラットフォーム対応キーボード統計ロガーです。

## セキュリティ保証

**機密情報は一切記録されません**

> オープンソースなので、セキュリティが不安な方はコードを確認して下さい。

- **キー押下回数のみ**をカウント（A: 45回、Space: 123回など）
- **実際のテキストは保存しない**（パスワード等の復元不可能）
- **入力順序や組み合わせは記録しない**（Ctrl+Cなどの情報なし）
- **メモリ内のみで動作**（ディスクへの一時保存なし）
- **ネットワーク通信なし**（データはマシン内にとどまる）

```rust
// 保存されるデータの例
{
    "A": 45,          // Aキーが45回押された
    "Space": 123,     // スペースキーが123回押された
    "Enter": 8        // Enterキーが8回押された
}
```

**メモリ使用量**: 約3MB（統計データは最大2.4KB、長時間使用でも増加しません(多分)）

## 機能

- クロスプラットフォーム対応（macOS・Windows）
- リアルタイムキー押下統計・CSVエクスポート
- 軽量・高速・インストール不要

## インストール

[リリースページ](https://github.com/okawak/key_logger/releases)からマシンに合ったのバイナリをダウンロードしてください。

- **macOS (Intel)**: `key_logger-macos-intel`
- **macOS (Apple Silicon)**: `key_logger-macos-apple`
- **Windows**: `key_logger-windows.exe`

## 使用方法

### macOS

```bash
./key_logger
```
**初回実行時**: システム環境設定 → プライバシー → アクセシビリティで権限を許可してください。

### Windows

```powershell
.\key_logger.exe
```

**初回実行時**: 管理者権限で実行してください。

**共通**: `Ctrl+C`で停止してCSV出力します。

**環境変数**:
- `KEY_LOGGER_OUTPUT_DIR`: 出力先ディレクトリ
- `RUST_LOG`: ログレベル (`error`, `warn`, `info`, `debug`)

## 出力

タイムスタンプ付きCSVファイル（例：`keylog_2025-07-23_14-30-00.csv`）

```csv
Key,Count
Space,245
E,189
T,156
A,134
```

## 開発者向け

```bash
# ビルド・テスト
cargo build --release
cargo test
cargo clippy
```

## ライセンス

MIT ライセンス

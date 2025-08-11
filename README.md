[![CI](https://github.com/okawak/key_logger/actions/workflows/ci.yml/badge.svg)](https://github.com/okawak/key_logger/actions/workflows/ci.yml) [![Release](https://github.com/okawak/key_logger/actions/workflows/release.yml/badge.svg)](https://github.com/okawak/key_logger/actions/workflows/release.yml) [![Versioning](https://github.com/okawak/key_logger/actions/workflows/version.yml/badge.svg)](https://github.com/okawak/key_logger/actions/workflows/version.yml) [![Security Audit](https://github.com/okawak/key_logger/actions/workflows/security.yml/badge.svg)](https://github.com/okawak/key_logger/actions/workflows/security.yml)

# Key Logger

シンプルなクロスプラットフォーム対応キーボード統計ロガーです。

## セキュリティ保証

**機密情報は一切記録されません(はず)**

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

**メモリ使用量**: 約3MB（統計データは最大2.4KB、長時間使用でも増加しない）

## 機能

- **クロスプラットフォーム対応**（macOS・Windows・Linux）
- **キー監視**（ポーリング処理）
- **軽量**・インストール不要
- **標準ログ**（`log`クレートを使用）
- **バッチ処理**（効率的なキー記録）

## インストール

[リリースページ](https://github.com/okawak/key_logger/releases)からお使いのプラットフォーム用のバイナリをダウンロードしてください。

- **macOS (Intel)**: `key_logger-macos-intel`
- **macOS (Apple Silicon)**: `key_logger-macos-apple`
- **Windows**: `key_logger-windows.exe`
- **Linux**: `key_logger-linux`

## 使用方法

### macOS
```bash
cd /path/to/downloaded/binary
chmod +x key_logger-macos-apple # Apple Siliconの場合
./key_logger-macos-apple
```
**初回実行時**: システム環境設定 → プライバシー → アクセシビリティで権限を許可してください。

### Windows
```powershell
.\key_logger.exe
```

### Linux
```bash
cd /path/to/downloaded/binary
chmod +x key_logger-linux
./key_logger-linux
```

**共通**: `Ctrl+C`で停止してCSV出力します。

**環境変数**:
- `KEY_LOGGER_OUTPUT_DIR`: 出力先ディレクトリ（省略時は現在のディレクトリ）
- `RUST_LOG`: ログレベル (`error`, `warn`, `info`, `debug`)

## 出力

タイムスタンプ付きCSVファイル（例：`keylog_2025-07-27_14-30-00.csv`）

```csv
Key,Count
Space,245
E,189
T,156
A,134
```

### 出力例

```
[INFO] Key Logger starting...
[INFO] Press Ctrl+C to stop and save statistics
[INFO] Output directory: (current working directory)
^C
[INFO] Received exit signal, saving statistics...
[INFO] Saving statistics...
[INFO] Statistics saved to: keylog_2025-07-27_14-30-00.csv
[INFO] Total key presses: 1247
[INFO] Unique keys pressed: 23
[INFO] Top 10 most pressed keys:
[INFO] 1. Space: 245
[INFO] 2. E: 189
[INFO] 3. T: 156
...
```

## 開発者向け

```bash
# ビルド・テスト
cargo build --release
cargo test
cargo clippy

# デバッグ実行
RUST_LOG=debug ./key_logger

# 特定ディレクトリに出力
KEY_LOGGER_OUTPUT_DIR=/tmp ./key_logger
```

## パフォーマンス最適化

- **静的文字列使用**: キー名に`&'static str`を使用してメモリ効率を向上
- **バッチ処理**: 複数キーを一度に処理
- **事前割り当て**: `HashMap`と`Vec`の容量を事前確保
- **メモリ効率**: u64を使用し大きな数値もサポート

## ライセンス

MIT ライセンス

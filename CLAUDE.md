# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

Rust workspace for HorrorGame.net (`hgn`) backend tools. The first crate is `ogp-generator`, a CLI tool that generates OGP (Open Graph Protocol) PNG images for game reviews. It is invoked via `Process::run()` from a Laravel queue worker, receives parameters as a JSON string argument, and writes PNG files directly to Laravel's `public/img/review/` directory on the same server.

## Workspace Structure

```
hgn_rust_tools/
├── Cargo.toml              # Workspace root (members = ["crates/*"], resolver = "2")
├── crates/
│   ├── ogp-generator/      # OGP image generation CLI tool
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs     # Entry point: arg parsing, env vars, file output
│   │   │   ├── image.rs    # SVG template rendering via resvg
│   │   │   └── hash.rs     # SHA-256 filename hashing
│   │   └── assets/
│   │       ├── review-template.svg
│   │       └── fonts/NotoSansJP-Bold.ttf
│   └── shared/             # Shared utilities (future)
└── docs/plan/              # Spec documents
```

> The workspace and crates do not exist yet — `docs/plan/1-1.ogp-service.md` is the implementation spec.

## Build & Run Commands

> **重要**: `cargo build` を実行する際は、必ず続けて `make install-dev` も実行すること。
> ビルドしたバイナリを `/usr/local/bin/hgn/ogp-generator` に反映させるため。

```bash
# Development build (specific crate)
cargo build -p ogp-generator

# Release build
cargo build -p ogp-generator --release

# Release build + /usr/local/bin/hgn/ へコピー（開発環境でのローカルインストール）
make install-dev

# Run tests for a specific crate
cargo test -p ogp-generator

# Run a single test
cargo test -p ogp-generator test_function_name

# Check for compile errors without producing a binary
cargo check -p ogp-generator

# Lint
cargo clippy -p ogp-generator

# Format
cargo fmt
```

## Running ogp-generator

```bash
OUTPUT_DIR={hgs_re3設置パス}/public/img/review \
FONT_PATH=/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc \
SVG_TEMPLATE_PATH=./crates/ogp-generator/assets/review-template.svg \
./target/release/ogp-generator \
  '{"review_id":1,"game_title_name":"バイオハザード RE:2","show_id":"huckle","total_score":95,"fear_meter":3,"score_story":4,"score_atmosphere":4,"score_gameplay":3,"has_spoiler":false}'
```

## System Dependencies (WSL2 Ubuntu)

Build environment only:
```bash
sudo apt install -y build-essential pkg-config libfontconfig1-dev fonts-noto-cjk
```

Production server (runtime only):
```bash
sudo apt install -y libfontconfig1 fonts-noto-cjk
```

## Key Design Decisions

- **Invocation**: Called via `Process::run([binary, json])` from Laravel queue worker — no HTTP server, no port.
- **SVG templating**: `assets/review-template.svg` uses `{{PLACEHOLDER}}` strings replaced via `str::replace` before passing to `resvg` for PNG rendering.
- **Filename hashing**: `review_id` is SHA-256 hashed to prevent sequential ID enumeration. Output: `{hex}.png`.
- **Output**: PNG written directly to Laravel's `public/img/review/` filesystem path (no S3/CDN).
- **Response**: Outputs JSON to stdout. Returns `{"ok": true, "filename": "{hash}.png"}` on success, `{"ok": false, "error": "..."}` on failure. Exit code 0/1.
- **resvg/usvg versions must match** — they are co-maintained in the same upstream repo.

## Environment Variables

| Variable | Example |
|----------|---------|
| `OUTPUT_DIR` | `{hgs_re3設置パス}/public/img/review` |
| `FONT_PATH` | `/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc` |
| `SVG_TEMPLATE_PATH` | `{hgs_re3設置パス}/resources/ogp/review-template.svg` |

These are passed explicitly by Laravel via `Process::run(..., env: [...])`. See `docs/plan/1-1.ogp-service.md` for details.

## CLI Contract

Accepts a single JSON argument. See `docs/plan/1-1.ogp-service.md` for the full schema.

Null handling:
- `total_score: null` → display `"-点"`
- axis score `null` → omit that axis from the image
- `has_spoiler: true` → show `【ネタバレあり】` badge top-left

## Deployment

Binary is deployed to `/usr/local/bin/ogp-generator` on the server via GitHub Actions (see `docs/plan/1-2.implementation-plan.md` Phase 7). No service process or systemd unit required — the binary is invoked per-job by Laravel's queue worker.

## Security — Public Repository

**このリポジトリはGitHub上でpublicです。** ドキュメントやコードに以下の情報を記載しないこと：

- サーバーの実際のディレクトリパス（例: `/var/www/html/hgs_re3/` → `{hgs_re3設置パス}` のようなプレースホルダーを使う）
- IPアドレス・ホスト名
- 認証情報・APIキー・秘密鍵
- 本番・STG環境の具体的なインフラ構成

環境依存の値はすべて `.env`（gitignore済み）または GitHub Secrets に置くこと。

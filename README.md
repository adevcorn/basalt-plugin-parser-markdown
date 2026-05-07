# basalt-plugin-parser-markdown

Basalt syntax parser plugin for Markdown (`.md`) files.

Provides tree-sitter-based syntax highlighting and semantic retrieval chunks (headings as chunks) for Markdown files via the Basalt parser plugin ABI.

## Installation

Install via the Basalt plugin registry, or manually:

```bash
curl -L https://github.com/adevcorn/basalt-plugin-parser-markdown/releases/latest/download/parser.wasm \
  -o ~/.config/basalt/parsers/md.wasm
```

## Building from source

```bash
rustup target add wasm32-wasip1
export CC_wasm32_wasip1=/path/to/clang
export AR_wasm32_wasip1=/path/to/llvm-ar
export CFLAGS_wasm32_wasip1="--sysroot=/path/to/wasi-sysroot -include wasi_dup_stub.h"
cargo build --release --target wasm32-wasip1
cp target/wasm32-wasip1/release/basalt_plugin_parser_markdown.wasm ~/.config/basalt/parsers/md.wasm
```

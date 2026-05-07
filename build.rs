fn main() {
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() != Ok("wasm32") { return; }
    // tree-sitter-md compiles into 4 archives: parser_block, parser_inline,
    // scanner_block, scanner_inline.
    for lib in ["parser_block", "parser_inline", "scanner_block", "scanner_inline"] {
        println!("cargo:rustc-link-arg=--whole-archive");
        println!("cargo:rustc-link-arg=-l{lib}");
        println!("cargo:rustc-link-arg=--no-whole-archive");
    }
}

// markdown-parser/src/lib.rs — tree-sitter Markdown parser WASM plugin for Basalt

use tree_sitter::{Language, Parser, Query, QueryCursor};

const SRC_OFFSET: usize = 1 * 1024 * 1024;
const OUT_OFFSET: usize = 6 * 1024 * 1024;
const MEMORY_BYTES: usize = 12 * 1024 * 1024;

const SCOPE_KEYWORD: u8 = 1;
const SCOPE_STRING: u8 = 2;
const SCOPE_FUNCTION: u8 = 6;
const SCOPE_TYPE: u8 = 5;
const SCOPE_COMMENT: u8 = 4;

static mut MEMORY: [u8; MEMORY_BYTES] = [0u8; MEMORY_BYTES];
static LANG_EXT: &[u8] = b"md\0";

extern "C" { fn tree_sitter_markdown() -> Language; }

#[no_mangle]
pub extern "C" fn basalt_lang() -> i32 {
    LANG_EXT.as_ptr() as i32
}

#[no_mangle]
pub unsafe extern "C" fn basalt_parse(
    src_ptr: i32, src_len: i32, out_ptr: i32, max_spans: i32,
) -> i32 {
    let src = std::slice::from_raw_parts(
        (MEMORY.as_ptr() as usize + src_ptr as usize) as *const u8,
        src_len as usize,
    );
    let out = std::slice::from_raw_parts_mut(
        (MEMORY.as_ptr() as usize + out_ptr as usize) as *mut u8,
        (max_spans as usize) * 12,
    );

    let lang = tree_sitter_markdown();
    let mut parser = Parser::new();
    if parser.set_language(lang).is_err() { return 0; }
    let Some(tree) = parser.parse(src, None) else { return 0; };

    let query_src = r#"
        (atx_heading) @keyword
        (fenced_code_block) @string
        (code_span) @string
        (emphasis) @function
        (strong_emphasis) @function
        (link_text) @type
        (block_quote) @comment
    "#;
    let Ok(query) = Query::new(lang, query_src) else { return 0; };
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, tree.root_node(), src);
    let cap_names = query.capture_names().to_vec();

    let mut count = 0usize;
    for m in matches {
        for cap in m.captures {
            if count >= max_spans as usize { break; }
            let scope_id = match cap_names[cap.index as usize].as_str() {
                "keyword"  => SCOPE_KEYWORD,
                "string"   => SCOPE_STRING,
                "function" => SCOPE_FUNCTION,
                "type"     => SCOPE_TYPE,
                "comment"  => SCOPE_COMMENT,
                _ => 0,
            };
            let offset = cap.node.start_byte() as u32;
            let length = (cap.node.end_byte() - cap.node.start_byte()) as u32;
            let base = count * 12;
            out[base..base+4].copy_from_slice(&offset.to_le_bytes());
            out[base+4..base+8].copy_from_slice(&length.to_le_bytes());
            out[base+8] = scope_id;
            out[base+9] = 0; out[base+10] = 0; out[base+11] = 0;
            count += 1;
        }
    }
    count as i32
}

#[no_mangle]
pub unsafe extern "C" fn basalt_retrieval_chunks(
    src_ptr: i32, src_len: i32, out_ptr: i32, max_chunks: i32,
) -> i32 {
    let src = std::slice::from_raw_parts(
        (MEMORY.as_ptr() as usize + src_ptr as usize) as *const u8,
        src_len as usize,
    );
    let out = std::slice::from_raw_parts_mut(
        (MEMORY.as_ptr() as usize + out_ptr as usize) as *mut u8,
        (max_chunks as usize) * 104,
    );

    let lang = tree_sitter_markdown();
    let mut parser = Parser::new();
    if parser.set_language(lang).is_err() { return 0; }
    let Some(tree) = parser.parse(src, None) else { return 0; };

    // Use ATX headings as retrieval chunks (h1–h6).
    let query_src = r#"(atx_heading (atx_h1_marker) heading_content: (_) @name.module) @chunk.module
                       (atx_heading (atx_h2_marker) heading_content: (_) @name.module) @chunk.module
                       (atx_heading (atx_h3_marker) heading_content: (_) @name.function) @chunk.function"#;
    let Ok(query) = Query::new(lang, query_src) else { return 0; };
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, tree.root_node(), src);
    let cap_names = query.capture_names().to_vec();

    let mut count = 0usize;
    for m in matches {
        if count >= max_chunks as usize { break; }
        let mut offset = None::<u32>;
        let mut length = None::<u32>;
        let mut kind = None::<&str>;
        let mut name = None::<&str>;
        for cap in m.captures {
            let cn = &cap_names[cap.index as usize];
            if let Some(k) = cn.strip_prefix("chunk.") {
                offset = Some(cap.node.start_byte() as u32);
                length = Some((cap.node.end_byte() - cap.node.start_byte()) as u32);
                kind = Some(k);
            } else if cn.starts_with("name.") {
                if let Ok(t) = cap.node.utf8_text(src) { name = Some(t.trim()); }
            }
        }
        let (Some(off), Some(len), Some(k)) = (offset, length, kind) else { continue };
        let label = if let Some(n) = name {
            let mut s = k.to_string(); s.push(' '); s.push_str(n); s
        } else { k.to_string() };
        let base = count * 104;
        out[base..base+4].copy_from_slice(&off.to_le_bytes());
        out[base+4..base+8].copy_from_slice(&len.to_le_bytes());
        let lbytes = label.as_bytes();
        let llen = lbytes.len().min(95);
        out[base+8..base+8+llen].copy_from_slice(&lbytes[..llen]);
        out[base+8+llen] = 0;
        count += 1;
    }
    count as i32
}

/// Markdown has no call sites.
#[no_mangle]
pub unsafe extern "C" fn basalt_call_sites(
    _src_ptr: i32, _src_len: i32, _out_ptr: i32, _max_sites: i32,
) -> i32 {
    0
}

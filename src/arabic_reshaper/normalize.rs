use alloc::string::String;

/// Post-processing to clean up common visual artifacts after reordering.
pub fn normalize_visual_rtl_output(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }

    // 1. Handle Step Markers (e.g., "1)", "2)"...)
    // We do this first to identify the "body" of the text.
    let (marker, body) = extract_trailing_step_marker(s);
    let body = body.trim();

    // 2. Single-pass normalization
    // We pre-allocate capacity to avoid re-allocations during the build.
    let mut result = String::with_capacity(body.len());

    // We use char_indices to allow us to look ahead in the string slice efficiently.
    let mut char_iter = body.char_indices().peekable();

    while let Some((idx, c)) = char_iter.next() {
        if c.is_whitespace() {
            // Check if we should skip this space

            // A) Collapse multiple spaces
            if result.ends_with(' ') {
                continue;
            }

            // B) Skip space if it's followed by a slash or preceded by one
            // We peek at the next character
            let next_char = char_iter.peek().map(|&(_, nc)| nc);
            if next_char == Some('/') || result.ends_with('/') {
                continue;
            }

            // C) Skip spaces around XML entities (&quot;)
            // Check ahead: is the next part "&quot;"?
            if body[idx..].starts_with(" &quot;") {
                continue;
            }
            // Check behind: did we just push "&quot;"?
            if result.ends_with("&quot;") {
                continue;
            }

            result.push(' ');
        } else {
            result.push(c);
        }
    }

    // 3. Final Assembly
    let trimmed_result = result.trim();
    if let Some(m) = marker {
        if trimmed_result.is_empty() {
            String::from(m)
        } else {
            // Concatenate: "1) ShapedText"
            let mut final_out = String::with_capacity(m.len() + 1 + trimmed_result.len());
            final_out.push_str(m);
            final_out.push(' ');
            final_out.push_str(trimmed_result);
            final_out
        }
    } else {
        String::from(trimmed_result)
    }
}

/// Detects trailing markers like "1)" through "9)" without allocating new strings.
fn extract_trailing_step_marker(s: &str) -> (Option<&str>, &str) {
    let trimmed = s.trim_end();

    // Static patterns to avoid formatting strings in a loop
    const MARKERS: [&str; 9] = ["1)", "2)", "3)", "4)", "5)", "6)", "7)", "8)", "9)"];

    for marker in MARKERS {
        if trimmed.ends_with(marker) {
            let prefix = &trimmed[..trimmed.len() - marker.len()];
            return (Some(marker), prefix);
        }
    }

    (None, s)
}

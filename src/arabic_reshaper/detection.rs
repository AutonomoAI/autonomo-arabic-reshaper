// ==== src/arabic_reshaper/detection.rs ====

pub fn contains_arabic_letters(s: &str) -> bool {
    s.chars().any(is_arabic_char)
}

/// Checks if a character is an Arabic letter or a shape-shifting character.
/// This specifically EXCLUDES Arabic-Indic numerals (which are LTR).
pub fn is_arabic_char(c: char) -> bool {
    let u = c as u32;
    let is_in_range = matches!(u,
        0x0600..=0x06FF | 0x0750..=0x077F | 0x08A0..=0x08FF |
        0xFB50..=0xFDFF | 0xFE70..=0xFEFF
    );
    // Arabic letters are RTL, but Arabic numerals (0x0660-0x0669) are LTR.
    is_in_range && !is_arabic_numeral(c)
}

pub fn is_bidi_mark(c: char) -> bool {
    matches!(c, '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}')
}

pub fn is_diacritic(c: char) -> bool {
    // Covers standard harakat (064B-0652), shadda, maddah, and superscript alef (0670)
    matches!(c, '\u{064B}'..='\u{065F}' | '\u{0670}')
}

pub fn is_arabic_numeral(c: char) -> bool {
    // 0660..0669: Arabic-Indic digits
    // 06F0..06F9: Extended Arabic-Indic digits (Persian/Urdu)
    matches!(c as u32, 0x0660..=0x0669 | 0x06F0..=0x06F9)
}

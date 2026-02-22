// ==== src/arabic_reshaper/shaping.rs ====

use alloc::string::String;
use alloc::vec::Vec;

use super::detection::is_diacritic;

/// Applies contextual Arabic shaping (isolated / initial / medial / final forms)
/// plus mandatory Lam-Alef ligatures. Diacritics are passed through unchanged.
pub fn apply_arabic_shaping(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut shaped = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // 1. Mandatory Lam-Alef ligatures (required for correct display)
        // TODO: does not handle diacritics between ل and alef variants (e.g. لَا)
        if c == 'ل' && i + 1 < chars.len() {
            let mut prev_connects = false;
            for j in (0..i).rev() {
                if is_diacritic(chars[j]) { continue; }
                prev_connects = connects_to_left(chars[j]);
                break;
            }

            let ligature = match chars[i + 1] {
                'ا' => Some(('\u{FEFB}', '\u{FEFC}')),
                'أ' => Some(('\u{FEF7}', '\u{FEF8}')),
                'إ' => Some(('\u{FEF9}', '\u{FEFA}')),
                'آ' => Some(('\u{FEF5}', '\u{FEF6}')),
                _   => None,
            };
            if let Some((iso, fin)) = ligature {
                //println!("Lam at {} → next char '{}' (U+{:04x})", i, chars[i + 1], chars[i + 1] as u32);
                //println!("prev_connects = {}", prev_connects);
                //println!("Chosen form: {}", if prev_connects { fin } else { iso });
                shaped.push(if prev_connects { fin } else { iso });
                i += 2;
                continue;
            }
        }

        // 2. Standard letter shaping
        if let Some((iso, fin, ini, med, connects_left)) = get_arabic_form(c) {
            let prev_connects = i > 0 && connects_to_left(chars[i - 1]);

            // Look ahead, skipping diacritics, to find the next base letter.
            // We check connects_left of the next letter — not just whether it's Arabic —
            // because right-only connectors (ا د ذ ر ز و ى) refuse a connection from the left,
            // so the current letter should take final form rather than medial.
            let mut next_connects = false;
            let mut j = i + 1;
            while j < chars.len() {
                if is_diacritic(chars[j]) {
                    j += 1;
                    continue;
                }
                // next_connects = get_arabic_form(chars[j]).map_or(false, |f| f.4);
                next_connects = get_arabic_form(chars[j]).is_some();

                break;
            }

            let form = match (prev_connects, connects_left && next_connects) {
                (false, false) => iso,
                (true,  false) => fin,
                (false, true)  => ini,
                (true,  true)  => med,
            };
            shaped.push(form);
        } else {
            // Non-Arabic letter or diacritic → pass through unchanged
            shaped.push(c);
        }

        i += 1;
    }

    shaped.into_iter().collect()
}

fn connects_to_left(c: char) -> bool {
    get_arabic_form(c).map_or(false, |f| f.4)
}

/// Returns shaping forms for base Arabic letters:
/// (Isolated, Final, Initial, Medial, ConnectsToLeft)
/// All mapped to Unicode Arabic Presentation Forms-A/B.
pub fn get_arabic_form(c: char) -> Option<(char, char, char, char, bool)> {
    match c {
        'ء' => Some(('\u{FE80}', '\u{FE80}', '\u{FE80}', '\u{FE80}', false)),
        'آ' => Some(('\u{FE81}', '\u{FE82}', '\u{FE81}', '\u{FE82}', false)),
        'أ' => Some(('\u{FE83}', '\u{FE84}', '\u{FE83}', '\u{FE84}', false)),
        'ؤ' => Some(('\u{FE85}', '\u{FE86}', '\u{FE85}', '\u{FE86}', false)),
        'إ' => Some(('\u{FE87}', '\u{FE88}', '\u{FE87}', '\u{FE88}', false)),
        'ئ' => Some(('\u{FE89}', '\u{FE8A}', '\u{FE8B}', '\u{FE8C}', true)),
        'ا' => Some(('\u{FE8D}', '\u{FE8E}', '\u{FE8D}', '\u{FE8E}', false)),
        'ب' => Some(('\u{FE8F}', '\u{FE90}', '\u{FE91}', '\u{FE92}', true)),
        'ة' => Some(('\u{FE93}', '\u{FE94}', '\u{FE93}', '\u{FE94}', false)),
        'ت' => Some(('\u{FE95}', '\u{FE96}', '\u{FE97}', '\u{FE98}', true)),
        'ث' => Some(('\u{FE99}', '\u{FE9A}', '\u{FE9B}', '\u{FE9C}', true)),
        'ج' => Some(('\u{FE9D}', '\u{FE9E}', '\u{FE9F}', '\u{FEA0}', true)),
        'ح' => Some(('\u{FEA1}', '\u{FEA2}', '\u{FEA3}', '\u{FEA4}', true)),
        'خ' => Some(('\u{FEA5}', '\u{FEA6}', '\u{FEA7}', '\u{FEA8}', true)),
        'د' => Some(('\u{FEA9}', '\u{FEAA}', '\u{FEA9}', '\u{FEAA}', false)),
        'ذ' => Some(('\u{FEAB}', '\u{FEAC}', '\u{FEAB}', '\u{FEAC}', false)),
        'ر' => Some(('\u{FEAD}', '\u{FEAE}', '\u{FEAD}', '\u{FEAE}', false)),
        'ز' => Some(('\u{FEAF}', '\u{FEB0}', '\u{FEAF}', '\u{FEB0}', false)),
        'س' => Some(('\u{FEB1}', '\u{FEB2}', '\u{FEB3}', '\u{FEB4}', true)),
        'ش' => Some(('\u{FEB5}', '\u{FEB6}', '\u{FEB7}', '\u{FEB8}', true)),
        'ص' => Some(('\u{FEB9}', '\u{FEBA}', '\u{FEBB}', '\u{FEBC}', true)),
        'ض' => Some(('\u{FEBD}', '\u{FEBE}', '\u{FEBF}', '\u{FEC0}', true)),
        'ط' => Some(('\u{FEC1}', '\u{FEC2}', '\u{FEC3}', '\u{FEC4}', true)),
        'ظ' => Some(('\u{FEC5}', '\u{FEC6}', '\u{FEC7}', '\u{FEC8}', true)),
        'ع' => Some(('\u{FEC9}', '\u{FECA}', '\u{FECB}', '\u{FECC}', true)),
        'غ' => Some(('\u{FECD}', '\u{FECE}', '\u{FECF}', '\u{FED0}', true)),
        'ف' => Some(('\u{FED1}', '\u{FED2}', '\u{FED3}', '\u{FED4}', true)),
        'ق' => Some(('\u{FED5}', '\u{FED6}', '\u{FED7}', '\u{FED8}', true)),
        'ك' => Some(('\u{FED9}', '\u{FEDA}', '\u{FEDB}', '\u{FEDC}', true)),
        'ل' => Some(('\u{FEDD}', '\u{FEDE}', '\u{FEDF}', '\u{FEE0}', true)),
        'م' => Some(('\u{FEE1}', '\u{FEE2}', '\u{FEE3}', '\u{FEE4}', true)),
        'ن' => Some(('\u{FEE5}', '\u{FEE6}', '\u{FEE7}', '\u{FEE8}', true)),
        'ه' => Some(('\u{FEE9}', '\u{FEEA}', '\u{FEEB}', '\u{FEEC}', true)),
        'و' => Some(('\u{FEED}', '\u{FEEE}', '\u{FEED}', '\u{FEEE}', false)),
        'ى' => Some(('\u{FEEF}', '\u{FEF0}', '\u{FEEF}', '\u{FEF0}', false)),
        'ي' => Some(('\u{FEF1}', '\u{FEF2}', '\u{FEF3}', '\u{FEF4}', true)),
        // TODO: extend with Persian/Urdu letters if needed (پ چ ژ گ ...)
        _ => None,
    }
}

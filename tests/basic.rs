// ==== tests/basic.rs ====

#[cfg(test)]
mod tests {
    use autonomo_arabic_reshaper::{
        apply_arabic_shaping,
        normalize_visual_rtl_output,
        reshape_arabic,
        contains_arabic_letters,
    };

    // ── Detection ────────────────────────────────────────────────────────────

    #[test]
    fn test_contains_arabic_detection() {
        assert!(!contains_arabic_letters("Just ASCII text"));
        assert!(!contains_arabic_letters(""));
        assert!(!contains_arabic_letters("12345 !@#$%"));
        assert!(contains_arabic_letters("مرحبا"));
        assert!(contains_arabic_letters("Hello مرحبا World"));
        assert!(contains_arabic_letters("ﻲﻓ")); // already shaped (presentation forms)
    }

    // ── Identity / passthrough ───────────────────────────────────────────────

    #[test]
    fn test_no_arabic_returns_input_unchanged() {
        assert_eq!(reshape_arabic("Hello World! 123"), "Hello World! 123");
        assert_eq!(reshape_arabic(""), "");
        assert_eq!(reshape_arabic("   "), "   ");
        assert_eq!(reshape_arabic("{TAG} <color=red>text</color>"), "{TAG} <color=red>text</color>");
    }

    // ── Shaping: presentation forms ─────────────────────────────────────────

    #[test]
    fn test_output_uses_presentation_forms() {
        let output = reshape_arabic("كتاب");
        let pf = 0xFE70u32..=0xFEFFu32;
        assert!(
            output.chars().any(|c| pf.contains(&(c as u32))),
            "Expected Arabic Presentation Forms in output, got: {output:?}"
        );
    }

    #[test]
    fn test_kitab_shaping_codepoints() {
        // كتاب (k-t-a-b)
        // ك: initial (prev=none, next=ت connects)        → FEDB
        // ت: final   (prev=ك connects, next=ا doesn't connect_left) → FE96
        // ا: final   (prev=ت connects, ا itself doesn't connect_left) → FE8E
        // ب: isolated (prev=ا which doesn't connect_left, next=none) → FE8F
        let shaped = apply_arabic_shaping("كتاب");
        let cps: Vec<u32> = shaped.chars().map(|c| c as u32).collect();
        assert_eq!(cps, vec![0xFEDB, 0xFE98, 0xFE8E, 0xFE8F],
            "Shaped codepoints wrong: {cps:04x?}");

        let output = reshape_arabic("كتاب");
        let out_cps: Vec<u32> = output.chars().map(|c| c as u32).collect();
        assert_eq!(out_cps, vec![0xFE8F, 0xFE8E, 0xFE98, 0xFEDB],
            "Visual-reversed codepoints wrong: {out_cps:04x?}");
    }
    // ── Lam-Alef ligatures ───────────────────────────────────────────────────

    #[test]
    fn test_lam_alef_isolated_ligature() {
        // Standalone لا → isolated ﻻ (U+FEFB), single character
        let out = reshape_arabic("لا");
        assert_eq!(out.chars().count(), 1, "Should collapse to one ligature char");
        assert_eq!(out.chars().next().unwrap() as u32, 0xFEFB,
            "Expected isolated ﻻ U+FEFB");
    }

    #[test]
    fn test_lam_alef_final_ligature_after_connector() {
        // قلا: ق connects → lam-alef gets final form FEFC
        // After visual reversal the ligature appears first
        let out = reshape_arabic("قلا");
        assert_eq!(out.chars().count(), 2, "Should be two chars: ligature + ق");
        let cps: Vec<u32> = out.chars().map(|c| c as u32).collect();
        assert_eq!(cps[0], 0xFEFC, "First char (visual) should be final ﻼ U+FEFC");
    }

    #[test]
    fn test_lam_hamza_alef_final_ligature() {
        // قلأ: ق connects → lam-hamza-alef gets final form FEF8
        let out = reshape_arabic("قلأ");
        let cps: Vec<u32> = out.chars().map(|c| c as u32).collect();
        assert_eq!(cps[0], 0xFEF8,
            "First char (visual) should be final ﻸ U+FEF8, got {:04x?}", cps);
    }

    #[test]
    fn test_lam_alef_with_hamza_below_isolated() {
        // Standalone لإ → isolated U+FEF9
        let out = reshape_arabic("لإ");
        assert_eq!(out.chars().count(), 1);
        assert_eq!(out.chars().next().unwrap() as u32, 0xFEF9,
            "Expected isolated ﻹ U+FEF9");
    }

    #[test]
    fn test_lam_alef_madda_isolated() {
        // Standalone لآ → isolated U+FEF5
        let out = reshape_arabic("لآ");
        assert_eq!(out.chars().count(), 1);
        assert_eq!(out.chars().next().unwrap() as u32, 0xFEF5,
            "Expected isolated ﻵ U+FEF5");
    }

    // ── Diacritics ───────────────────────────────────────────────────────────

    #[test]
    fn test_diacritics_pass_through() {
        // كَتَب with fatha diacritics — shaping should still work, diacritics preserved
        let input = "كَتَب";
        let shaped = apply_arabic_shaping(input);
        // Diacritics (U+064E fatha) should still be present
        assert!(shaped.chars().any(|c| c as u32 == 0x064E),
            "Fatha diacritic should be preserved");
        // Base letters should still be shaped into presentation forms
        let pf = 0xFE70u32..=0xFEFFu32;
        assert!(shaped.chars().any(|c| pf.contains(&(c as u32))),
            "Presentation forms should be applied even with diacritics");
    }

    // ── Tag / entity / escape protection ────────────────────────────────────

    #[test]
    fn test_curly_tag_protected() {
        let out = reshape_arabic("مرحبا {PAWN_name}!");
        assert!(out.contains("{PAWN_name}"), "Curly tag must survive intact");
    }

    #[test]
    fn test_angle_tag_protected() {
        let out = reshape_arabic("النص <color=#FF0000>أحمر</color> هنا");
        assert!(out.contains("<color=#FF0000>"), "Opening tag must survive");
        assert!(out.contains("</color>"), "Closing tag must survive");
    }

    #[test]
    fn test_square_tag_protected() {
        let out = reshape_arabic("الرابط [i]مائل[/i]");
        assert!(out.contains("[i]") && out.contains("[/i]"), "Square tags must survive");
    }

    #[test]
    fn test_xml_entity_protected() {
        let out = reshape_arabic("قل &quot;مرحبا&quot;");
        assert!(out.contains("&quot;"), "XML entity must survive intact");
    }

    #[test]
    fn test_mixed_tags_and_arabic() {
        let input = "مرحبا {PAWN_name}! كيف حالك؟";
        let out = reshape_arabic(input);
        assert!(out.contains("{PAWN_name}"), "Tag must be intact");
        // Tags should not glue to Arabic text
        assert!(!out.contains("{PAWN_name}ﻚ"), "Tag must not glue to Arabic");
    }

    // ── Newline preservation ─────────────────────────────────────────────────

    #[test]
    fn test_literal_backslash_n_preserved() {
        let input = "سطر أول\\nسطر ثاني";
        let out = reshape_arabic(input);
        assert_eq!(out.matches("\\n").count(), 1, "Literal \\n must be preserved");
        assert_eq!(out.split("\\n").count(), 2);
    }

    #[test]
    fn test_real_newline_preserved() {
        let input = "سطر أول\nسطر ثاني";
        let out = reshape_arabic(input);
        assert_eq!(out.matches('\n').count(), 1, "Real newline must be preserved");
        assert_eq!(out.split('\n').count(), 2);
    }

    #[test]
    fn test_multiple_newlines_preserved() {
        let input = "أولاً\n\nثانياً";
        let out = reshape_arabic(input);
        assert_eq!(out.split('\n').count(), 3, "Double newline should produce 3 segments");
    }

    // ── Normalization ────────────────────────────────────────────────────────

    #[test]
    fn test_normalize_double_spaces_collapsed() {
        let input = "كلمة  أخرى"; // two spaces
        let out = reshape_arabic(input);
        assert!(!out.contains("  "), "Double spaces should be collapsed");
    }

    #[test]
    fn test_normalize_step_marker_moved_to_front() {
        // Trailing "1)" on an Arabic line should move to front after normalization
        let visual = "ﺢﺘﻓﺍ Steam 1)"; // already in visual form
        let normalized = normalize_visual_rtl_output(visual);
        assert!(normalized.starts_with("1)"), "Step marker should move to front, got: {normalized:?}");
    }

    #[test]
    fn test_normalize_slash_spacing() {
        let out = normalize_visual_rtl_output("يمين / يسار");
        assert!(!out.contains(" / "), "Spaces around slash should be removed, got: {out:?}");
    }

    // ── RTL/LTR boundary spacing ─────────────────────────────────────────────

    #[test]
    fn test_no_gluing_arabic_english() {
        let out = reshape_arabic("افتح RimWorld الآن");
        // After visual reversal: "الآن RimWorld افتح" style
        // English and Arabic words must have spaces between them
        assert!(!out.contains("RimWorldﺍ") && !out.contains("ﺍRimWorld"),
            "Arabic and English must not glue: {out:?}");
    }

    #[test]
    fn test_english_command_protected() {
        let out = reshape_arabic("أضف \"-popupwindow\" للخيارات");
        assert!(out.contains("-popupwindow"), "Command flag must survive");
    }

    // ── Storyteller (regression) ──────────────────────────────────────────────

    #[test]
    fn test_storyteller_settings_exact() {
        // "إعدادات راوي القصص" — three words, no tags
        // Expected visual output (shaped + reversed):
        // ﺺﺼﻘﻟﺍ ﻱﻭﺍﺭ ﺕﺍﺩﺍﺪﻋﺇ
        let out = reshape_arabic("إعدادات راوي القصص");
        assert_eq!(out, "ﺺﺼﻘﻟﺍ ﻱﻭﺍﺭ ﺕﺍﺩﺍﺪﻋﺇ",
            "Storyteller mismatch: {out:?}");
    }

    // ── Edge cases ───────────────────────────────────────────────────────────

    #[test]
    fn test_single_arabic_letter() {
        let out = reshape_arabic("ب");
        // Isolated form of ب = U+FE8F
        assert_eq!(out.chars().next().unwrap() as u32, 0xFE8F,
            "Single ب should be isolated form FE8F");
    }

    #[test]
    fn test_arabic_punctuation_passthrough() {
        // Arabic question mark ؟ and comma ، should survive
        let out = reshape_arabic("كيف حالك؟");
        assert!(out.contains('؟'), "Arabic question mark must survive");
    }

    #[test]
    fn test_numbers_in_arabic_text() {
        // Numbers should not be reversed
        let out = reshape_arabic("الرقم 42 صحيح");
        assert!(out.contains("42"), "Numbers must survive intact");
    }

    #[test]
    fn test_already_shaped_input_not_double_processed() {
        // Feeding presentation-form Arabic back in — contains_arabic_letters returns true
        // for presentation ranges, shaping should still produce valid output (not crash)
        let pre_shaped = "ﺐﺘﻜﻳ"; // already shaped يكتب
        let out = reshape_arabic(pre_shaped);
        assert!(!out.is_empty(), "Should handle pre-shaped input without crashing");
    }
}

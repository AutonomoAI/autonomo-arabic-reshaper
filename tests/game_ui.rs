// ==== tests/game_ui.rs ====

#[cfg(test)]
mod tests {
    use autonomo_arabic_reshaper::reshape_arabic;

    // ── Storyteller ──────────────────────────────────────────────────────────

    #[test]
    fn test_storyteller_settings() {
        let out = reshape_arabic("إعدادات راوي القصص");
        assert_eq!(out, "ﺺﺼﻘﻟﺍ ﻱﻭﺍﺭ ﺕﺍﺩﺍﺪﻋﺇ");
    }

    // ── Texture compression warning ──────────────────────────────────────────

    const INPUT_TEXTURE_AR: &str =
        "يتطلب تغيير إعدادات ضغط النسيج إعادة تشغيل. سيضيع التقدم غير المحفوظ.\n\nالمتابعة؟";

    #[test]
    fn test_texture_warning_line1() {
        let out = reshape_arabic(INPUT_TEXTURE_AR);
        let line1 = out.split('\n').next().unwrap();
        // "يتطلب تغيير إعدادات ضغط النسيج إعادة تشغيل. سيضيع التقدم غير المحفوظ."
        // Both sentences are on line 1, visually reversed
        assert!(line1.contains("ﺞﻴﺴﻨﻟﺍ ﻂﻐﺿ ﺕﺍﺩﺍﺪﻋﺇ ﺮﻴﻴﻐﺗ ﺐﻠﻄﺘﻳ"),
            "First sentence (requires restart) missing: {line1:?}");
        assert!(line1.contains("ﻞﻴﻐﺸﺗ ﺓﺩﺎﻋﺇ"),
            "Restart phrase missing: {line1:?}");
        assert!(line1.contains("ﻅﻮﻔﺤﻤﻟﺍ ﺮﻴﻏ ﻡﺪﻘﺘﻟﺍ ﻊﻴﻀﻴﺳ"),
            "Second sentence (unsaved progress) missing: {line1:?}");
    }

    #[test]
    fn test_texture_warning_last_line() {
        let out = reshape_arabic(INPUT_TEXTURE_AR);
        let last = out.split('\n').last().unwrap();
        assert!(last.contains("؟ﺔﻌﺑﺎﺘﻤﻟﺍ"),
            "Continue? line mismatch: {last:?}");
    }

    #[test]
    fn test_texture_warning_newlines_preserved() {
        let out = reshape_arabic(INPUT_TEXTURE_AR);
        assert_eq!(out.split('\n').count(), 3,
            "Expected 3 segments (blank line between): got {}", out.split('\n').count());
    }

    // ── Tutorial text ────────────────────────────────────────────────────────

    const INPUT_TUTORIAL_AR: &str =
        "‏لتشغيل RimWorld في وضع ملء الشاشة بدون حدود:\n\n1) افتح Steam\n2) اذهب إلى تبويب \"المكتبة\" في الأعلى\n3) ابحث عن RimWorld في القائمة وانقر بزر الماوس الأيمن عليه\n4) اختر \"خصائص…\"\n5) أضف \"-popupwindow\" بدون علامات اقتباس إلى \"خيارات التشغيل\" في تبويب \"عام\"\n6) أعد تشغيل اللعبة\n7) إذا تم بشكل صحيح، فلن ترى هذه النافذة\n\nتلميح: استخدم مفتاح Windows + Shift + يسار/يمين لتحريك النافذة بين شاشات العرض الخاصة بك.‏‎";

    #[test]
    fn test_tutorial_step1_moved() {
        let out = reshape_arabic(INPUT_TUTORIAL_AR);
        // "1) افتح Steam" → after reshape: "1) Steam ﺢﺘﻓﺍ"
        // step marker is already at front (normalize_visual_rtl_output moves trailing markers)
        assert!(out.contains("1) Steam ﺢﺘﻓﺍ"),
            "Step 1 should be: '1) Steam ﺢﺘﻓﺍ', got: {out:?}");
    }

    #[test]
    fn test_tutorial_popupwindow_protected() {
        let out = reshape_arabic(INPUT_TUTORIAL_AR);
        assert!(out.contains("-popupwindow"), "Command flag must survive");
    }

    #[test]
    fn test_tutorial_windows_shift_protected() {
        let out = reshape_arabic(INPUT_TUTORIAL_AR);
        assert!(out.contains("Windows + Shift +"), "Key combo must survive intact");
    }

    #[test]
    fn test_tutorial_rimworld_spaced_from_arabic() {
        let out = reshape_arabic(INPUT_TUTORIAL_AR);
        assert!(out.contains("ﻲﻓ RimWorld"), "Arabic and RimWorld must be spaced");
        assert!(!out.contains("RimWorldﻲﻓ") && !out.contains("ﻲﻓRimWorld"),
            "Arabic and English must not glue");
    }

    #[test]
    fn test_tutorial_header_phrase() {
        let out = reshape_arabic(INPUT_TUTORIAL_AR);
        assert!(out.contains("ﺩﻭﺪﺣ ﻥﻭﺪﺑ ﺔﺷﺎﺸﻟﺍ ﺀﻠﻣ ﻊﺿﻭ ﻲﻓ RimWorld ﻞﻴﻐﺸﺘﻟ"),
            "Header phrase mismatch: {out:?}");
    }

    #[test]
    fn test_tutorial_newline_count_preserved() {
        let out = reshape_arabic(INPUT_TUTORIAL_AR);
        let expected = INPUT_TUTORIAL_AR.chars().filter(|&c| c == '\n').count();
        let got = out.chars().filter(|&c| c == '\n').count();
        assert_eq!(got, expected, "Newline count must be preserved");
    }

    // ── Mixed English/Arabic with XML-style quotes ───────────────────────────

    #[test]
    fn test_mixed_quote_content_protected() {
        let input = "افتح \"Steam\" الآن";
        let out = reshape_arabic(input);
        assert!(out.contains("\"Steam\""), "Quoted English word must survive: {out:?}");
    }

    // ── Multi-tag line ───────────────────────────────────────────────────────

    #[test]
    fn test_multiple_tags_in_one_line() {
        let input = "الاسم: {PAWN_name} الصحة: <color=red>{health}</color>";
        let out = reshape_arabic(input);
        assert!(out.contains("{PAWN_name}"), "PAWN_name tag must survive");
        assert!(out.contains("<color=red>"), "color tag must survive");
        assert!(out.contains("{health}"), "health tag must survive");
        assert!(out.contains("</color>"), "closing tag must survive");
    }

    // ── Hint line (colon + mixed) ────────────────────────────────────────────

    #[test]
    fn test_hint_colon_line() {
        // Last line of tutorial: "تلميح: استخدم مفتاح Windows + Shift + يسار/يمين ..."
        let input = "تلميح: استخدم مفتاح Windows + Shift + يسار/يمين لتحريك النافذة.";
        let out = reshape_arabic(input);
        assert!(out.contains("Windows + Shift +"), "Key combo must survive");
        assert!(!out.is_empty());
    }
}

# arabic-reshaper

**Arabic text shaper + visual RTL reverser** — turns logical Arabic text into presentation forms and visually reordered LTR-compatible strings.

This crate is specifically designed for **game modding** environments (especially RimWorld, Unity UIs, and similar LTR-biased engines) where:

- Text must be shaped into isolated/initial/medial/final forms + mandatory Lam-Alef ligatures
- Visual right-to-left reordering is needed (because the engine renders LTR)
- Game tags (`{PAWN_nameDef}`, `<color>`, `[i]`, XML entities `&quot;`, escapes `\n` / `\t`) must be **protected** and **not reversed**
- Long LTR runs (commands, proper names, paths, key combos) must stay in logical order
- Quoted ASCII spans (`"-popupwindow"`, `"General"`, `"Steam"`) are treated as atomic LTR tokens and never split by reversal
- Common visual artifacts (glued words, bad spacing around quotes/slashes, trailing step markers) are cleaned up

### Primary use cases

- Processing **JSON**, **JSONL**, and **XML** localization files (RimWorld's `Keyed`, `Defs`, mod strings)
- Feeding translated Arabic text into game engines that lack proper bidi/shaping support
- Part of automated translation pipelines for RTL languages

It will *probably* work well on any Arabic-containing document, but the heuristics (tag protection, LTR chunk detection, step-marker moving, spacing fixes) are tuned for **game UI strings**.

### Proven in production

This is a crucial component of the **[Autonomous RimWorld Translator](https://github.com/AutonomoAI/rimworld-autonomous-translator)** — an autonomous, self-correcting localization pipeline that translates RimWorld + mods into 30+ languages.

It powered the fully playable **[Rimworld-Arabic](https://github.com/BetterRimworlds/Rimworld-Arabic)** mod — the first high-quality, AI-generated Arabic translation of RimWorld.

### Features

- Contextual Arabic shaping (presentation forms A/B)
- Mandatory Lam-Alef ligatures (all four alef variants: ا أ إ آ)
- Per-line visual RTL reversal (logical → visual order)
- Strict protection of game tags: `{…}`, `<…>`, `[…]`, XML entities, backslash escapes
- Detection & protection of LTR chunks: plain ASCII runs (RimWorld, Steam, key combos) and **quoted ASCII spans** (`"-popupwindow"`, `"General"`) as atomic tokens
- Smart spacing insertion between RTL/LTR boundaries (avoids glued words like `ﻲﻓRimWorld`)
- Post-processing normalizations:
  - Clean spaces around `&quot;`
  - Fix slashes in mixed tokens (`يمين/ يسار` → `يمين/يسار`)
  - Collapse double spaces
  - Move trailing step markers (`1)` … `9)`) to line start
- Fast range-based Arabic detection
- Handles literal `\n` vs real newlines (preserves format from game files)
- `no_std` + `alloc` compatible

### Quick start

```rust
use arabic_reshaper::reshape_arabic;

fn main() {
    let input = "مرحبا {PAWN_nameDef}! افتح RimWorld الآن 1)";
    let reshaped = reshape_arabic(input);
    println!("{}", reshaped);
    // ≈ "1) !{PAWN_nameDef} ﺢﺘﻓﺍ RimWorld ﺎﺒﺤﺮﻣ"
    // (visual RTL order + shaped letters + protected tag + moved marker)
}
```

### Known limitations / TODOs

- Persian/Urdu letters (پ چ ژ گ …) not yet in the shaping table

### Status

- `no_std` + `alloc` compatible
- Full test suite covering shaping, ligatures, tag protection, normalization, and real game UI strings

### Roadmap

- Persian/Urdu letter extensions
- Diacritic-aware ligature detection
- Full Unicode bidi algorithm fallback
- WASM demo

### License

MIT License

Part of the Autonomo AI Platform ecosystem — building autonomous tools for game worlds.

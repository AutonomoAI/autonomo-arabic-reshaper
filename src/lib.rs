// ==== src/lib.rs ====

#![no_std]
extern crate alloc;

pub use arabic_reshaper::{
    contains_arabic_letters,
    reshape_arabic,
    apply_arabic_shaping,        // Add for tests debugging...
    manual_rtl_for_unity,        // Add for tests debugging...
    normalize_visual_rtl_output, // Add for tests debugging...
};

mod arabic_reshaper {
    pub mod detection;
    pub mod shaping;
    pub mod reordering;
    pub mod normalize;

    pub use detection::contains_arabic_letters;
    pub use reordering::reshape_arabic;
    pub use shaping::apply_arabic_shaping;                  // ← re-export
    pub use reordering::manual_rtl_for_unity;               // ← optional
    pub use normalize::normalize_visual_rtl_output;         // ← optional
}

//! Paradox Mod Translator - AI-powered translation tool for Paradox game mods.

pub mod config;
pub mod preprocess;
pub mod translate;
pub mod postprocess;
pub mod utils;

pub mod error;

// Re-export commonly used types
pub use error::{TranslationError, Result};

#[cfg(test)]
mod tests {
    // Keep existing test structure for now
    #[test]
    fn it_works() {
        // Simple placeholder test
        assert_eq!(2 + 2, 4);
    }
}

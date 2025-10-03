//! Language definitions for localization system

/// Supported languages for the application
#[derive(Debug, Clone, Copy)]
pub enum Language {
    English,
    Ukrainian,
    Russian,
    Chinese,
}

impl Language {
    /// Get all supported languages
    pub fn all() -> &'static [Language] {
        &[Language::English, Language::Ukrainian, Language::Russian, Language::Chinese]
    }

    /// Get human-readable name of the language
    pub fn name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Ukrainian => "Українська",
            Language::Russian => "Русский",
            Language::Chinese => "中文",
        }
    }

    /// Get language code (ISO 639-1)
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Ukrainian => "uk",
            Language::Russian => "ru", 
            Language::Chinese => "zh",
        }
    }
}
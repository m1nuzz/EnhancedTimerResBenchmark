//! Localization module for multilingual support
//!
//! This module provides internationalization support for all UI elements
//! in the timer resolution benchmark tool.

use crate::ui::language::Language;
pub use crate::ui::localization_key::LocalizationKey;

/// Localization system for multilingual support
pub struct Localization {
    pub language: Language,
}

impl Localization {
    /// Create a new localization instance for the specified language
    pub fn new(language: Language) -> Self {
        Self { language }
    }
    
    /// Get localized string for a given key
    pub fn get(&self, key: LocalizationKey) -> &'static str {
        match self.language {
            Language::English => key.get_english(),
            Language::Ukrainian => key.get_ukrainian(),
            Language::Russian => key.get_russian(),
            Language::Chinese => key.get_chinese(),
        }
    }
    
    /// Format working directory string based on selected language
    pub fn get_working_dir(&self, path: &str) -> String {
        match self.language {
            Language::English => format!("рџ“‚ Working Directory: {}", path),
            Language::Ukrainian => format!("рџ“‚ Р РѕР±РѕС‡Р° РґРёСЂРµРєС‚РѕСЂС–СЏ: {}", path),
            Language::Russian => format!("рџ“‚ Р Р°Р±РѕС‡Р°СЏ РґРёСЂРµРєС‚РѕСЂРёСЏ: {}", path),
            Language::Chinese => format!("рџ“‚ е·ҐдЅњз›®еЅ•: {}", path),
        }
    }
    
    /// Format Windows version string based on selected language
    pub fn get_windows_version(&self, info: &str) -> String {
        match self.language {
            Language::English => format!("рџ–ҐпёЏ Windows Version: {}", info),
            Language::Ukrainian => format!("рџ–ҐпёЏ Р’РµСЂСЃС–СЏ Windows: {}", info),
            Language::Russian => format!("рџ–ҐпёЏ Р’РµСЂСЃС–СЏ Windows: {}", info),
            Language::Chinese => format!("рџ–ҐпёЏ Windows з‰€жњ¬: {}", info),
        }
    }
    
    /// Format CPU information string based on selected language
    pub fn get_cpu(&self, cpu: &str) -> String {
        match self.language {
            Language::English => format!("рџ’» CPU: {}", cpu),
            Language::Ukrainian => format!("рџ’» РџСЂРѕС†РµСЃРѕСЂ: {}", cpu),
            Language::Russian => format!("рџ’» РџСЂРѕС†РµСЃСЃРѕСЂ: {}", cpu),
            Language::Chinese => format!("рџ’» CPU: {}", cpu),
        }
    }
    
    /// Format range information based on selected language
    pub fn get_range(&self, low: f64, high: f64) -> String {
        match self.language {
            Language::English => format!("Range: [{:.4}, {:.4}] ms", low, high),
            Language::Ukrainian => format!("Р”С–Р°РїР°Р·РѕРЅ: [{:.4}, {:.4}] РјСЃ", low, high),
            Language::Russian => format!("Р”РёР°РїР°Р·РѕРЅ: [{:.4}, {:.4}] РјСЃ", low, high),
            Language::Chinese => format!("иЊѓе›ґ: [{:.4}, {:.4}] жЇ«з§’", low, high),
        }
    }
    
    /// Format current best value information based on selected language
    pub fn get_current_best(&self, value: f64, score: f64) -> String {
        match self.language {
            Language::English => format!("Current best: {:.4} ms (score={:.4})", value, score),
            Language::Ukrainian => format!("РџРѕС‚РѕС‡РЅРёР№ РЅР°Р№РєСЂР°С‰РёР№: {:.4} РјСЃ (РѕС†С–РЅРєР°={:.4})", value, score),
            Language::Russian => format!("РўРµРєСѓС‰РёР№ Р»СѓС‡С€РёР№: {:.4} РјСЃ (РѕС†РµРЅРєР°={:.4})", value, score),
            Language::Chinese => format!("еЅ“е‰ЌжњЂдЅі: {:.4} жЇ«з§’ (е€†ж•°={:.4})", value, score),
        }
    }
    
    /// Format optimal value recommendation based on selected language
    pub fn get_optimal_value(&self, value: f64) -> String {
        match self.language {
            Language::English => format!("вњ… RECOMMENDED VALUE: {:.4} ms", value),
            Language::Ukrainian => format!("вњ… Р Р•РљРћРњР•РќР”РћР’РђРќР• Р—РќРђР§Р•РќРќРЇ: {:.4} РјСЃ", value),
            Language::Russian => format!("вњ… Р Р•РљРћРњР•РќР”РЈР•РњРћР• Р—РќРђР§Р•РќРР•: {:.4} РјСЃ", value),
            Language::Chinese => format!("вњ… жЋЁиЌђеЂј: {:.4} жЇ«з§’", value),
        }
    }
    
    /// Format optimal recommendation command based on selected language
    pub fn get_optimal_recommendation(&self, resolution: i32) -> String {
        match self.language {
            Language::English => format!("SetTimerResolution.exe --resolution {} --no-console", resolution),
            Language::Ukrainian => format!("SetTimerResolution.exe --resolution {} --no-console", resolution),
            Language::Russian => format!("SetTimerResolution.exe --resolution {} --no-console", resolution),
            Language::Chinese => format!("SetTimerResolution.exe --resolution {} --no-console", resolution),
        }
    }
    
    /// Format rank information based on selected language
    pub fn get_rank(&self, rank: usize) -> String {
        match self.language {
            Language::English => format!("Rank {}", rank),
            Language::Ukrainian => format!("Р РµР№С‚РёРЅРі {}", rank),
            Language::Russian => format!("Р РµР№С‚РёРЅРі {}", rank),
            Language::Chinese => format!("жЋ’еђЌ {}", rank),
        }
    }
    
    /// Format iterations with kappa information based on selected language
    pub fn get_iterations_with_kappa(&self, iteration: usize, value: f64, kappa: f64) -> String {
        match self.language {
            Language::English => format!("рџЋЇ Iteration {}: {:.4} ms (kappa={:.2})", iteration, value, kappa),
            Language::Ukrainian => format!("рџЋЇ Р†С‚РµСЂР°С†С–СЏ {}: {:.4} РјСЃ (kappa={:.2})", iteration, value, kappa),
            Language::Russian => format!("рџЋЇ РС‚РµСЂР°С†РёСЏ {}: {:.4} РјСЃ (kappa={:.2})", iteration, value, kappa),
            Language::Chinese => format!("рџЋЇ иї­д»Ј {}: {:.4} жЇ«з§’ (kappa={:.2})", iteration, value, kappa),
        }
    }
    
    /// Format phase 1 information based on selected language
    pub fn get_phase1(&self, count: usize) -> String {
        match self.language {
            Language::English => format!("[INIT] Phase 1: Initialization ({} points)", count),
            Language::Ukrainian => format!("[INIT] Р¤Р°Р·Р° 1: Р†РЅС–С†С–Р°Р»С–Р·Р°С†С–СЏ ({} С‚РѕС‡РєРё)", count),
            Language::Russian => format!("[INIT] Р¤Р°Р·Р° 1: РРЅРёС†РёР°Р»РёР·Р°С†РёСЏ ({} С‚РѕС‡РєРё)", count),
            Language::Chinese => format!("[INIT] й¶ж®µ 1: е€ќе§‹еЊ– ({} з‚№)", count),
        }
    }
    
    /// Format point information based on selected language
    pub fn get_point_info(&self, current: usize, total: usize, resolution: f64) -> String {
        match self.language {
            Language::English => format!("  [POINT] {}/{}: {:.4} ms", current, total, resolution),
            Language::Ukrainian => format!("  [POINT] {}/{}: {:.4} РјСЃ", current, total, resolution),
            Language::Russian => format!("  [POINT] {}/{}: {:.4} ms", current, total, resolution),
            Language::Chinese => format!("  [POINT] {}/{}: {:.4} жЇ«з§’", current, total, resolution),
        }
    }
    
    /// Format measurement information with runs and samples based on selected language
    pub fn get_measurement_with_runs(&self, resolution: f64, runs: usize, samples: i32) -> String {
        match self.language {
            Language::English => format!("    [TEST] Measurement {:.4} ms ({} runs x {} samples)...", resolution, runs, samples),
            Language::Ukrainian => format!("    [TEST] Р’РёРјС–СЂСЋРІР°РЅРЅСЏ {:.4} РјСЃ ({} Р·Р°РїСѓСЃРєС–РІ x {} РІРёР±С–СЂРѕРє)...", resolution, runs, samples),
            Language::Russian => format!("    [TEST] РР·РјРµСЂРµРЅРёРµ {:.4} ms ({} РїСЂРѕРіРѕРЅРѕРІ x {} РІС‹Р±РѕСЂРѕРє)...", resolution, runs, samples),
            Language::Chinese => format!("    [TEST] жµ‹й‡Џ {:.4} жЇ«з§’ ({} ж¬ЎиїђиЎЊ x {} ж ·жњ¬)...", resolution, runs, samples),
        }
    }
    
    /// Format measurement statistics based on selected language
    pub fn get_measurement_stats(&self, mean: f64, p95: f64, mad: f64, outliers: usize) -> String {
        match self.language {
            Language::English => format!("       μ={:.4} ms, p95={:.4} ms, MAD={:.4} ms, outliers={}", mean, p95, mad, outliers),
            Language::Ukrainian => format!("       μ={:.4} мс, p95={:.4} мс, MAD={:.4} мс, викидів={}", mean, p95, mad, outliers),
            Language::Russian => format!("       μ={:.4} ms, p95={:.4} ms, MAD={:.4} ms, выбросов={}", mean, p95, mad, outliers),
            Language::Chinese => format!("       μ={:.4} 毫秒, p95={:.4} 毫秒, MAD={:.4} 毫秒, 异常值={}", mean, p95, mad, outliers),
        }
    }
    
    /// Format MeasureSleep.exe error message based on selected language
    pub fn get_measure_sleep_error(&self, error: &str) -> String {
        match self.language {
            Language::English => format!("    вќЊ Error running MeasureSleep.exe: {}", error),
            Language::Ukrainian => format!("    вќЊ РџРѕРјРёР»РєР° Р·Р°РїСѓСЃРєСѓ MeasureSleep.exe: {}", error),
            Language::Russian => format!("    вќЊ РћС€РёР±РєР° Р·Р°РїСѓСЃРєР° MeasureSleep.exe: {}", error),
            Language::Chinese => format!("    вќЊ иїђиЎЊ MeasureSleep.exe й”™иЇЇ: {}", error),
        }
    }
    
    /// Format join error message based on selected language
    pub fn get_join_error(&self, error: &str) -> String {
        match self.language {
            Language::English => format!("    вќЊ Join error: {}", error),
            Language::Ukrainian => format!("    вќЊ РџРѕРјРёР»РєР° РїСЂРёС”РґРЅР°РЅРЅСЏ: {}", error),
            Language::Russian => format!("    вќЊ РћС€РёР±РєР° join: {}", error),
            Language::Chinese => format!("    вќЊ Join й”™иЇЇ: {}", error),
        }
    }
    
    /// Format timeout error message based on selected language
    pub fn get_timeout_error(&self) -> String {
        match self.language {
            Language::English => "    вќЊ MeasureSleep.exe timeout (>30s)".to_string(),
            Language::Ukrainian => "    вќЊ РўР°Р№Рј-Р°СѓС‚ MeasureSleep.exe (>30СЃ)".to_string(),
            Language::Russian => "    вќЊ РўР°Р№РјР°СѓС‚ MeasureSleep.exe (>30СЃ)".to_string(),
            Language::Chinese => "    вќЊ MeasureSleep.exe и¶…ж—¶ (>30з§’)".to_string(),
        }
    }
    
    /// Format keep current indicator based on selected language
    pub fn get_keep_current(&self) -> String {
        match self.language {
            Language::English => " (current)".to_string(),
            Language::Ukrainian => " (РїРѕС‚РѕС‡РЅРµ)".to_string(),
            Language::Russian => " (С‚РµРєСѓС‰РµРµ)".to_string(),
            Language::Chinese => " (еЅ“е‰Ќ)".to_string(),
        }
    }
    
    /// Format enter new value prompt based on selected language
    pub fn get_enter_new_value(&self) -> String {
        match self.language {
            Language::English => "Enter new value: ".to_string(),
            Language::Ukrainian => "Р’РІРµРґС–С‚СЊ РЅРѕРІРµ Р·РЅР°С‡РµРЅРЅСЏ: ".to_string(),
            Language::Russian => "Р’РІРµРґРёС‚Рµ РЅРѕРІРѕРµ Р·РЅР°С‡РµРЅРёРµ: ".to_string(),
            Language::Chinese => "иѕ“е…Ґж–°еЂј: ".to_string(),
        }
    }
    
    /// Format exit prompt based on selected language
    pub fn get_exit_prompt(&self) -> String {
        match self.language {
            Language::English => "Press Enter to exit...".to_string(),
            Language::Ukrainian => "РќР°С‚РёСЃРЅС–С‚СЊ Enter РґР»СЏ РІРёС…РѕРґСѓ...".to_string(),
            Language::Russian => "РќР°Р¶РјРёС‚Рµ Enter РґР»СЏ РІС‹С…РѕРґР°...".to_string(),
            Language::Chinese => "жЊ‰ Enter йЂЂе‡є...".to_string(),
        }
    }
}

/// Language selection function that allows users to choose their preferred language
pub fn select_language() -> Language {
    use std::io::{self, Write};
    use crate::ui::language::Language;
    
    println!("\n🌍 Select Language / Виберіть мову / Выберите язык / 选择语言");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let languages = Language::all();
    for (i, lang) in languages.iter().enumerate() {
        println!("{}. {}", i + 1, lang.name());
    }
    
    print!("\nSelect language (1-{}): ", languages.len());
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    let choice = input.trim().parse::<usize>().unwrap_or(1);
    let index = choice.saturating_sub(1).min(languages.len() - 1);
    
    languages[index]
}

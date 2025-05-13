use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// 形態素解析の依存関係を一時的に削除し、ダミー実装を提供

#[derive(Debug, Serialize, Deserialize)]
pub struct WordInfo {
    text: String,
    pos: String,
}

pub fn create_tokenizer() -> Result<(), Box<dyn std::error::Error>> {
    // ダミー実装
    Ok(())
}

pub fn split_text_into_bunsetsu(text: String) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // 簡易的な実装: スペースや句読点で分割
    let mut phrases = Vec::new();
    let mut current_phrase = String::new();
    
    for c in text.chars() {
        current_phrase.push(c);
        
        if c == ' ' || c == '、' || c == '。' || c == '!' || c == '?' || c == '！' || c == '？' {
            if !current_phrase.is_empty() {
                phrases.push(current_phrase.clone());
                current_phrase.clear();
            }
        }
    }
    
    // 最後の文節を追加
    if !current_phrase.is_empty() {
        phrases.push(current_phrase);
    }
    
    Ok(phrases)
}

pub fn analyze_text(text: String) -> Result<Vec<WordInfo>, Box<dyn std::error::Error>> {
    // 簡易的な実装: 単語ごとに分割する代わりに文字ごとに分割
    let words: Vec<WordInfo> = text
        .chars()
        .map(|c| WordInfo {
            text: c.to_string(),
            pos: "未知語".to_string(),
        })
        .collect();
    
    Ok(words)
}

pub fn analyze_text_stats(text: String) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // 簡易的な統計情報
    let chars_count = text.chars().count();
    let words_count = text.split_whitespace().count();
    let unique_words_count = text.split_whitespace().collect::<HashSet<_>>().len();
    
    let stats = serde_json::json!({
        "total_chars": chars_count,
        "total_words": words_count,
        "unique_words": unique_words_count,
        "reading_time": (words_count as f64 * 0.3).ceil() as i32 // 1単語あたり0.3秒で計算
    });
    
    Ok(stats)
} 
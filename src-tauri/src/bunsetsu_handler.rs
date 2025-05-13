use lindera::dictionary::{DictionaryKind};
use lindera::mode::Mode;
use lindera::tokenizer::Tokenizer;
use serde::{Deserialize, Serialize};
use std::sync::Once;
use std::collections::HashSet;

static INIT: Once = Once::new();
static mut TOKENIZER: Option<Tokenizer> = None;

pub fn create_tokenizer() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        if TOKENIZER.is_none() {
            // 最新バージョンでの初期化方法
            // 1. 辞書読み込み
            let dictionary = lindera::dictionary::load_dictionary_from_kind(DictionaryKind::IPADIC)?;
            
            // 2. セグメンターの作成
            let segmenter = lindera::segmenter::Segmenter::new(Mode::Normal, dictionary, None);
            
            // 3. トークナイザー作成
            TOKENIZER = Some(Tokenizer::new(segmenter));
        }
        Ok(())
    }
}

fn get_tokenizer() -> &'static Tokenizer {
    unsafe {
        if TOKENIZER.is_none() {
            create_tokenizer().expect("Failed to initialize tokenizer");
        }
        TOKENIZER.as_ref().unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WordInfo {
    text: String,
    pos: String,
}

// 品詞を判定する関数
fn is_separator_pos(pos: &str) -> bool {
    pos.starts_with("助詞") || 
    pos.starts_with("助動詞") || 
    pos.starts_with("記号")
}

pub fn split_text_into_bunsetsu(text: String) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let tokenizer = get_tokenizer();
    // &strが期待されているので、Stringの参照を渡す
    let tokens = tokenizer.tokenize(&text)?;
    
    let mut phrases = Vec::new();
    let mut current_phrase = String::new();
    
    for mut token in tokens {
        // トークンのテキストを追加
        current_phrase.push_str(&token.text);
        
        // details()メソッドを使って品詞情報を取得
        let details = token.details();
        
        // 最初の要素が品詞情報
        let pos = if !details.is_empty() {
            details[0]
        } else {
            "未知語"
        };
        
        // 文節の区切りになる品詞かチェック
        if is_separator_pos(pos) {
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
    let tokenizer = get_tokenizer();
    // &strが期待されているので、Stringの参照を渡す
    let tokens = tokenizer.tokenize(&text)?;
    
    let mut words = Vec::new();
    
    for mut token in tokens {
        // details()メソッドを使って品詞情報を取得
        let details = token.details();
        
        // 最初の要素が品詞情報
        let pos = if !details.is_empty() {
            details[0].to_string()
        } else {
            "未知語".to_string()
        };
        
        words.push(WordInfo {
            text: token.text.to_string(),
            pos,
        });
    }
    
    Ok(words)
}

pub fn analyze_text_stats(text: String) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let tokenizer = get_tokenizer();
    // &strが期待されているので、Stringの参照を渡す
    let tokens = tokenizer.tokenize(&text)?;
    
    // ユニークな単語数を事前に計算
    let mut unique = HashSet::new();
    for token in &tokens {
        unique.insert(token.text.to_string());
    }
    let unique_count = unique.len();
    
    let stats = serde_json::json!({
        "total_chars": text.chars().count(),
        "total_words": tokens.len(),
        "unique_words": unique_count,
        "reading_time": (tokens.len() as f64 * 0.3).ceil() as i32 // 1単語あたり0.3秒で計算
    });
    
    Ok(stats)
} 
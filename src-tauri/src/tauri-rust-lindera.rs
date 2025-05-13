use std::error::Error;
use std::fmt;
use lindera::tokenizer::{Tokenizer, TokenizerConfig};
use lindera::dictionary::{DictionaryConfig, DictionaryKind};
use serde::{Serialize, Deserialize};

// カスタムエラー型
#[derive(Debug)]
pub struct BunsetsuError(String);

impl fmt::Display for BunsetsuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "文節解析エラー: {}", self.0)
    }
}

impl Error for BunsetsuError {}

// 単語情報を格納する構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct WordInfo {
    pub surface: String,
    pub pos: String,
    pub pos_detail: Option<String>,
    pub pronunciation: Option<String>,
}

// 形態素解析器の初期化
pub fn create_tokenizer() -> Result<Tokenizer, Box<dyn Error>> {
    // 辞書設定 - IPADICを使用
    let dictionary = DictionaryConfig {
        kind: Some(DictionaryKind::IPADIC),
        path: None,
    };

    // トークナイザー設定
    let config = TokenizerConfig {
        dictionary,
        user_dictionary: None,
        mode: lindera::tokenizer::Mode::Normal,
    };

    // トークナイザーの作成
    let tokenizer = Tokenizer::with_config(config)?;
    Ok(tokenizer)
}

// テキストを文節に分割する関数
pub fn split_text_into_bunsetsu(text: String) -> Result<Vec<String>, Box<dyn Error>> {
    if text.is_empty() {
        return Ok(vec![]);
    }

    // トークナイザーの作成
    let tokenizer = create_tokenizer()?;
    
    // テキストを形態素解析
    let tokens = tokenizer.tokenize(&text)?;
    
    // 形態素情報から文節を構築
    let mut bunsetsu = Vec::new();
    let mut current_bunsetsu = String::new();
    let mut has_content_word = false;
    let mut has_particle = false;
    
    for token in &tokens {
        let surface = token.text.to_string();
        let pos = token.detail.clone().unwrap_or_default().pos.unwrap_or_default();
        
        // 文節の区切りを判定
        match pos.as_str() {
            // 助詞、助動詞の場合は現在の文節に追加
            "助詞" | "助動詞" => {
                current_bunsetsu.push_str(&surface);
                has_particle = true;
                
                // 助詞/助動詞の後で区切る場合が多い
                if has_content_word && has_particle {
                    if !current_bunsetsu.is_empty() {
                        bunsetsu.push(current_bunsetsu.clone());
                        current_bunsetsu.clear();
                    }
                    has_content_word = false;
                    has_particle = false;
                }
            },
            // 句読点や記号の場合
            "記号" => {
                current_bunsetsu.push_str(&surface);
                if !current_bunsetsu.is_empty() {
                    bunsetsu.push(current_bunsetsu.clone());
                    current_bunsetsu.clear();
                }
                has_content_word = false;
                has_particle = false;
            },
            // 接続詞は単独で文節を形成
            "接続詞" => {
                if !current_bunsetsu.is_empty() {
                    bunsetsu.push(current_bunsetsu.clone());
                    current_bunsetsu.clear();
                }
                current_bunsetsu.push_str(&surface);
                bunsetsu.push(current_bunsetsu.clone());
                current_bunsetsu.clear();
                has_content_word = false;
                has_particle = false;
            },
            // 内容語（名詞、動詞、形容詞など）
            "名詞" | "動詞" | "形容詞" | "副詞" => {
                // 前の文節に助詞があり、新しい内容語が始まる場合は文節を区切る
                if has_content_word && has_particle && !current_bunsetsu.is_empty() {
                    bunsetsu.push(current_bunsetsu.clone());
                    current_bunsetsu.clear();
                    has_particle = false;
                }
                current_bunsetsu.push_str(&surface);
                has_content_word = true;
            },
            // その他の品詞
            _ => {
                current_bunsetsu.push_str(&surface);
                has_content_word = true;
            }
        }
        
        // 文節が長すぎる場合は強制的に区切る（読みやすさのため）
        if current_bunsetsu.chars().count() > 15 && has_content_word {
            bunsetsu.push(current_bunsetsu.clone());
            current_bunsetsu.clear();
            has_content_word = false;
            has_particle = false;
        }
    }
    
    // 最後の文節があれば追加
    if !current_bunsetsu.is_empty() {
        bunsetsu.push(current_bunsetsu);
    }
    
    Ok(bunsetsu)
}

// トークン情報を取得する関数（詳細分析用）
pub fn analyze_text(text: String) -> Result<Vec<WordInfo>, Box<dyn Error>> {
    let tokenizer = create_tokenizer()?;
    let tokens = tokenizer.tokenize(&text)?;
    
    let mut word_infos = Vec::new();
    
    for token in tokens {
        let detail = token.detail.clone().unwrap_or_default();
        
        word_infos.push(WordInfo {
            surface: token.text.to_string(),
            pos: detail.pos.unwrap_or_default(),
            pos_detail: detail.pos_detail1,
            pronunciation: detail.pronunciation,
        });
    }
    
    Ok(word_infos)
}

// テキストの簡易分析（単語数、文字数など）
pub fn analyze_text_stats(text: String) -> Result<serde_json::Value, Box<dyn Error>> {
    let tokenizer = create_tokenizer()?;
    let tokens = tokenizer.tokenize(&text)?;
    
    let total_tokens = tokens.len();
    let char_count = text.chars().count();
    
    let mut noun_count = 0;
    let mut verb_count = 0;
    let mut adj_count = 0;
    let mut particle_count = 0;
    
    for token in tokens {
        if let Some(detail) = token.detail {
            if let Some(pos) = detail.pos {
                match pos.as_str() {
                    "名詞" => noun_count += 1,
                    "動詞" => verb_count += 1,
                    "形容詞" => adj_count += 1,
                    "助詞" => particle_count += 1,
                    _ => {}
                }
            }
        }
    }
    
    // 統計情報をJSONで返す
    let stats = serde_json::json!({
        "char_count": char_count,
        "token_count": total_tokens,
        "noun_count": noun_count,
        "verb_count": verb_count,
        "adj_count": adj_count,
        "particle_count": particle_count,
    });
    
    Ok(stats)
}

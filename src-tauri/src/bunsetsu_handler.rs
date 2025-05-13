use lindera::dictionary::DictionaryKind;
use lindera::mode::Mode;
use lindera::tokenizer::Tokenizer;
use serde::{Deserialize, Serialize};
use std::sync::Once;

static INIT: Once = Once::new();
static mut TOKENIZER: Option<Tokenizer> = None;

pub fn create_tokenizer() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        if TOKENIZER.is_none() {
            let dictionary =
                lindera::dictionary::load_dictionary_from_kind(DictionaryKind::IPADIC)?;
            let segmenter = lindera::segmenter::Segmenter::new(Mode::Normal, dictionary, None);
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

// トークン情報を保持する構造体
struct TokenInfo {
    text: String,
    features: Vec<String>,
}

impl TokenInfo {
    fn pos(&self) -> &str {
        self.features.get(0).map(|s| s.as_str()).unwrap_or("未知語")
    }

    fn pos_detail_1(&self) -> Option<&str> {
        self.features.get(1).map(|s| s.as_str())
    }

    fn pos_detail_2(&self) -> Option<&str> {
        self.features.get(2).map(|s| s.as_str())
    }

    fn conjugation_form(&self) -> Option<&str> {
        self.features.get(5).map(|s| s.as_str())
    }

    fn conjugation_type(&self) -> Option<&str> {
        self.features.get(4).map(|s| s.as_str())
    }

    fn base_form(&self) -> Option<&str> {
        self.features.get(6).map(|s| s.as_str())
    }

    fn reading(&self) -> Option<&str> {
        self.features.get(7).map(|s| s.as_str())
    }
}

// 助詞の詳細な分類と文節境界判定
fn check_particle_boundary(current: &TokenInfo, next: &TokenInfo) -> bool {
    let particle_type = current.pos_detail_1().unwrap_or("");
    let particle_text = current.text.as_str();
    let next_pos = next.pos();

    match particle_type {
        "格助詞" => {
            match particle_text {
                "の" => {
                    // 「の」は連体修飾なので基本的に区切らない
                    // ただし、後ろが用言（動詞・形容詞・形容動詞）の場合は区切る
                    matches!(next_pos, "動詞" | "形容詞" | "形容動詞" | "助動詞")
                }
                "と" => {
                    // 引用の「と」は区切らない
                    if let Some(base) = next.base_form() {
                        if matches!(base, "いう" | "言う" | "思う" | "考える" | "する" | "なる")
                        {
                            return false;
                        }
                    }
                    true
                }
                // その他の格助詞（が、を、に、で、へ、から、まで、より）は区切る
                _ => true,
            }
        }
        "係助詞" | "副助詞" => {
            // は、も、こそ、さえ、すら、など は必ず区切る
            true
        }
        "接続助詞" => {
            // ので、のに、から、ため、たり、つつ、ながら など
            // 基本的に区切るが、一部例外あり
            match particle_text {
                "て" | "で" => {
                    // 「て」「で」は次が補助動詞の場合は区切らない
                    if next_pos == "動詞" {
                        if let Some(detail) = next.pos_detail_1() {
                            return detail != "非自立";
                        }
                    }
                    next_pos != "助動詞"
                }
                _ => true,
            }
        }
        "終助詞" => {
            // か、ね、よ、な、ぞ、ぜ など
            // 文末なので必ず区切る
            true
        }
        "連体化" => {
            // 「の」（形式名詞的用法）
            false
        }
        "並立助詞" => {
            // や、か、とか など
            true
        }
        _ => {
            // その他の助詞
            // 基本的に区切るが、連続する助詞は区切らない
            next_pos != "助詞" && next_pos != "助動詞"
        }
    }
}

// 用言（動詞・形容詞・形容動詞）の活用形による文節境界判定
fn check_conjugation_boundary(current: &TokenInfo, next: &TokenInfo) -> bool {
    let conjugation = current.conjugation_form().unwrap_or("");
    let next_pos = next.pos();

    match conjugation {
        "終止形" | "基本形" => {
            // 文末なので基本的に区切る
            // ただし、次が助動詞・助詞の場合は例外
            !matches!(next_pos, "助動詞" | "助詞")
        }
        "連体形" => {
            // 連体修飾なので区切らない
            false
        }
        "連用形" => {
            // 連用形は文脈による
            match next_pos {
                "助動詞" => false, // 〜している、〜してある など
                "動詞" => {
                    // 複合動詞かどうか判定
                    if let Some(detail) = next.pos_detail_1() {
                        detail == "非自立" // 補助動詞なら区切らない
                    } else {
                        true // 独立した動詞なら区切る
                    }
                }
                _ => true,
            }
        }
        "仮定形" => {
            // 〜ば、〜たら の形
            matches!(next_pos, "助詞")
        }
        "命令形" => {
            // 文末なので区切る
            true
        }
        _ => false,
    }
}

// 助動詞の文節境界判定
fn check_auxiliary_boundary(current: &TokenInfo, next: &TokenInfo) -> bool {
    let aux_text = current.text.as_str();
    let next_pos = next.pos();

    // 助動詞の種類による判定
    match aux_text {
        "いる" | "ある" | "おる" => {
            // 補助動詞的な助動詞
            matches!(next_pos, "助詞" | "記号")
        }
        "ない" | "ぬ" | "ん" => {
            // 否定の助動詞
            if next_pos == "助詞" {
                // 「〜ないで」「〜ないから」など
                if let Some(particle) = next.text.as_str().chars().next() {
                    matches!(particle, 'で' | 'か' | 'の')
                } else {
                    true
                }
            } else {
                matches!(next_pos, "記号")
            }
        }
        "たい" | "たがる" => {
            // 希望の助動詞
            matches!(next_pos, "助詞" | "記号")
        }
        "れる" | "られる" | "せる" | "させる" => {
            // 受身・使役の助動詞
            !matches!(next_pos, "助動詞")
        }
        _ => {
            // その他の助動詞
            matches!(next_pos, "助詞" | "記号")
        }
    }
}

// 文節境界を判定するメイン関数
fn is_bunsetsu_boundary(current: &TokenInfo, next: &TokenInfo) -> bool {
    let curr_pos = current.pos();
    let next_pos = next.pos();

    // 記号の処理
    if curr_pos == "記号" {
        match current.text.as_str() {
            "、" | "。" | "！" | "？" | "…" => return true,
            "」" | "』" | "）" | "】" => return true,
            "「" | "『" | "（" | "【" => return false,
            _ => {}
        }
    }

    // 品詞別の詳細な判定
    match curr_pos {
        "助詞" => check_particle_boundary(current, next),
        "動詞" | "形容詞" | "形容動詞" => check_conjugation_boundary(current, next),
        "助動詞" => check_auxiliary_boundary(current, next),
        "接続詞" => true,  // 接続詞は独立した文節
        "感動詞" => true,  // 感動詞も独立
        "接頭詞" => false, // 接頭詞は次と結合
        "名詞" => {
            // 名詞の後の処理
            match next_pos {
                "助詞" => false,   // 名詞＋助詞は一つの文節
                "接尾詞" => false, // 名詞＋接尾詞も結合
                "名詞" => {
                    // 複合名詞の判定
                    if let Some(detail) = current.pos_detail_1() {
                        if detail == "固有名詞" {
                            // 固有名詞の後は区切ることが多い
                            if let Some(next_detail) = next.pos_detail_1() {
                                !matches!(next_detail, "接尾" | "非自立")
                            } else {
                                true
                            }
                        } else {
                            false // 一般名詞は結合
                        }
                    } else {
                        false
                    }
                }
                _ => false,
            }
        }
        "副詞" => {
            // 副詞は基本的に独立
            !matches!(next_pos, "助詞")
        }
        "連体詞" => {
            // 連体詞は次の名詞と結合
            false
        }
        _ => false,
    }
}

pub fn split_text_into_bunsetsu(text: String) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    eprintln!("入力テキスト: {}", text);

    let tokenizer = get_tokenizer();
    let mut tokens = tokenizer.tokenize(&text)?;

    // トークンから情報を抽出
    let mut token_infos = Vec::new();
    eprintln!("\n--- トークン情報 ---");
    for (i, token) in tokens.iter_mut().enumerate() {
        let features: Vec<String> = token.details().iter().map(|s| s.to_string()).collect();
        let token_info = TokenInfo {
            text: token.text.to_string(),
            features: features.clone(),
        };

        // 簡潔なログ出力（v2形式）
        eprint!("[{}]「{}」{}・", i, token_info.text, token_info.pos());
        if let Some(detail) = token_info.pos_detail_1() {
            eprint!("{}", detail);
        }
        if token_info.pos() == "動詞" || token_info.pos() == "形容詞" {
            if let Some(conj) = token_info.conjugation_form() {
                eprint!("・{}", conj);
            }
        }
        eprintln!();

        token_infos.push(token_info);
    }

    let mut phrases = Vec::new();
    let mut current_phrase = String::new();

    for i in 0..token_infos.len() {
        let info = &token_infos[i];
        current_phrase.push_str(&info.text);

        // 次のトークンがある場合、文節境界を判定
        if i < token_infos.len() - 1 {
            let next_info = &token_infos[i + 1];

            let is_boundary = is_bunsetsu_boundary(info, next_info);
            eprintln!(
                "境界判定: \"{}\" -> \"{}\" = {}",
                info.text, next_info.text, is_boundary
            );

            if is_boundary {
                if !current_phrase.is_empty() {
                    eprintln!("文節確定: \"{}\"", current_phrase);
                    phrases.push(current_phrase.clone());
                    current_phrase.clear();
                }
            }
        }
    }

    // 最後の文節を追加
    if !current_phrase.is_empty() {
        eprintln!("最後の文節: \"{}\"", current_phrase);
        phrases.push(current_phrase);
    }

    eprintln!("\n最終結果: {:?}", phrases);

    Ok(phrases)
}

// テスト用のmain関数（必要に応じてコメントアウトまたは削除）
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bunsetsu_split() -> Result<(), Box<dyn std::error::Error>> {
        let text = "人間は文章を読む時、滑らかに文字を読んでいる訳ではなく、「１点を見つめる」という事と「高速に視線を移動する」という事を繰り返しています。".to_string();
        let bunsetsu = split_text_into_bunsetsu(text)?;

        for b in &bunsetsu {
            print!("{} / ", b);
        }
        println!();

        assert!(!bunsetsu.is_empty());
        Ok(())
    }
}

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod bunsetsu_handler;

use bunsetsu_handler::{split_text_into_bunsetsu};
// command属性マクロをインポート
use tauri::command;

// 文節分割のコマンド
#[command]
fn split_bunsetsu(text: String) -> Result<Vec<String>, String> {
    split_text_into_bunsetsu(text).map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // 起動時にlinderaトークナイザが初期化できるか確認
            #[cfg(debug_assertions)]
            {
                match bunsetsu_handler::create_tokenizer() {
                    Ok(_) => println!("Lindera tokenizer initialized successfully"),
                    Err(e) => println!("Warning: Lindera initialization error: {}", e),
                }
            }
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        // .plugin(tauri_plugin_window::init())  // 互換性の問題があるため削除
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            split_bunsetsu
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
import { useState, useEffect } from "react";
import "./styles.css";
import BunpitsuReader from "./BunpitsuReader";
// import { invoke } from "@tauri-apps/api/core"; // 未使用なのでコメントアウト

import { open, save } from "@tauri-apps/plugin-dialog";
import { readTextFile, writeFile } from "@tauri-apps/plugin-fs";

function App() {
  const [inputText, setInputText] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [errorMessage, setErrorMessage] = useState("");

  // サンプルテキスト
  const sampleText = `人間は文章を読む時、滑らかに文字を読んでいる訳ではなく、「１点を見つめる」という事と「高速に視線を移動する」という事を繰り返しています。１点を見つめる事を固視と呼び、高速に視線を移動する事を跳躍運動（サッカード）と呼びます。文章の改行時には行末から次の行頭までの距離があるため、改行が多い程読むのに時間がかかります。しかし、印象の観点から言えば１行あたりの文字数が少ない方が好まれると言われています。おそらく文字数が多いと情報量が多くて疲れる印象になり、抵抗感が生まれてしまうためだと思います。`;

  // コンポーネント初期化時にサンプルテキストをセット
  useEffect(() => {
    setInputText(sampleText);
  }, []);

  // ファイルを開く処理
  const handleOpenFile = async () => {
    try {
      // ファイル選択ダイアログを表示
      const selected = await open({
        filters: [
          {
            name: "テキスト",
            extensions: ["txt"],
          },
        ],
      });

      if (selected) {
        setIsLoading(true);
        // ファイルを読み込み
        const content = await readTextFile(selected);
        setInputText(content);
        setErrorMessage("");
      }
    } catch (err) {
      setErrorMessage(`ファイルを開けませんでした: ${err}`);
    } finally {
      setIsLoading(false);
    }
  };

  // ファイルを保存する処理
  const handleSaveFile = async () => {
    try {
      // 保存ダイアログを表示
      const filePath = await save({
        filters: [
          {
            name: "テキスト",
            extensions: ["txt"],
          },
        ],
      });

      if (filePath) {
        // ファイルに保存
        // 文字列をエンコードしてUint8Array型に変換
        const encoder = new TextEncoder();
        const data = encoder.encode(inputText);
        await writeFile(filePath, data);
        setErrorMessage("");
      }
    } catch (err) {
      setErrorMessage(`ファイルを保存できませんでした: ${err}`);
    }
  };

  return (
    <div className="container">
      <h1>文節リーダー</h1>

      {/* ファイル操作ボタン */}
      <div className="file-buttons">
        <button onClick={handleOpenFile} disabled={isLoading}>
          ファイルを開く
        </button>
        <button onClick={handleSaveFile} disabled={isLoading}>
          保存
        </button>
      </div>

      {/* エラーメッセージ */}
      {errorMessage && <div className="error-message">{errorMessage}</div>}

      {/* テキスト入力エリア */}
      <div className="input-area">
        <label htmlFor="input-text">文章を入力または貼り付け:</label>
        <textarea
          id="input-text"
          value={inputText}
          onChange={(e) => setInputText(e.target.value)}
          placeholder="ここに文章を入力してください..."
          disabled={isLoading}
        />
      </div>

      {/* 文節リーダーコンポーネント */}
      <BunpitsuReader text={inputText} />
    </div>
  );
}

export default App;

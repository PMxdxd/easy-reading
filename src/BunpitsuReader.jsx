import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

function BunpitsuReader({ text }) {
  const [phrases, setPhrases] = useState([]);
  const [textStats, setTextStats] = useState(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isReading, setIsReading] = useState(false);
  const [currentPhraseIndex, setCurrentPhraseIndex] = useState(0);
  const [phraseSpeed, setPhraseSpeed] = useState(300);
  const [fontSize, setFontSize] = useState(24);
  const [error, setError] = useState("");
  const [showStats, setShowStats] = useState(false);

  // テキストが変更されたら文節分割を実行
  useEffect(() => {
    if (!text.trim()) {
      setPhrases([]);
      setTextStats(null);
      return;
    }

    const splitText = async () => {
      setIsLoading(true);
      setError("");

      try {
        // Rustバックエンドの文節分割処理を呼び出し
        const result = await invoke("split_bunsetsu", { text });
        setPhrases(result);
      } catch (err) {
        setError(`文節の分割に失敗しました: ${err}`);
        // バックアップとして単純な分割を行う
        const simpleResult = splitTextSimple(text);
        setPhrases(simpleResult);
      } finally {
        setIsLoading(false);
      }
    };

    splitText();
  }, [text]);

  // バックアップ用の簡易文節分割（Rustが使えない場合）
  const splitTextSimple = (text) => {
    if (!text) return [];

    // 簡易的な文節区切りのパターン
    const patterns = [
      "は",
      "が",
      "を",
      "に",
      "へ",
      "で",
      "と",
      "より",
      "から",
      "まで",
      "の",
      "や",
      "など",
      "ので",
      "のに",
      "けど",
      "から",
      "ため",
      "たり",
      "だり",
      "ます",
      "です",
      "ました",
      "でした",
      "ません",
      "ない",
      "たい",
      "、",
      "。",
      "！",
      "？",
      "「",
      "」",
      "（",
      "）",
    ];

    let phrases = [];
    let currentPhrase = "";

    for (let i = 0; i < text.length; i++) {
      currentPhrase += text[i];

      if (
        patterns.includes(text[i]) ||
        currentPhrase.length > 10 ||
        i === text.length - 1
      ) {
        if (currentPhrase.trim()) {
          phrases.push(currentPhrase);
        }
        currentPhrase = "";
      }
    }

    if (currentPhrase.trim()) {
      phrases.push(currentPhrase);
    }

    return phrases;
  };

  // 文節ごとのハイライト表示の制御
  useEffect(() => {
    let interval;

    if (isReading && phrases.length > 0) {
      interval = setInterval(() => {
        setCurrentPhraseIndex((prev) => {
          const next = prev + 1;
          if (next >= phrases.length) {
            setIsReading(false);
            return 0;
          }
          return next;
        });
      }, phraseSpeed);
    }

    return () => clearInterval(interval);
  }, [isReading, phrases, phraseSpeed]);

  // 読書開始
  const startReading = () => {
    setCurrentPhraseIndex(0);
    setIsReading(true);
  };

  // 読書停止
  const stopReading = () => {
    setIsReading(false);
  };

  // 次の文節へ
  const nextPhrase = () => {
    if (currentPhraseIndex < phrases.length - 1) {
      setCurrentPhraseIndex(currentPhraseIndex + 1);
    }
  };

  // 前の文節へ
  const prevPhrase = () => {
    if (currentPhraseIndex > 0) {
      setCurrentPhraseIndex(currentPhraseIndex - 1);
    }
  };

  // 統計情報の表示切り替え
  const toggleStats = () => {
    setShowStats(!showStats);
  };

  return (
    <div className="bunpitsu-reader">
      {/* コントロールパネル */}
      <div className="controls">
        <div className="speed-control">
          <label htmlFor="phrase-speed">
            表示速度: {phraseSpeed}ms
            <input
              id="phrase-speed"
              type="range"
              min="100"
              max="1000"
              step="50"
              value={phraseSpeed}
              onChange={(e) => setPhraseSpeed(parseInt(e.target.value))}
              disabled={isLoading}
            />
          </label>
        </div>

        <div className="font-control">
          <label htmlFor="font-size">
            フォントサイズ: {fontSize}px
            <input
              id="font-size"
              type="range"
              min="16"
              max="48"
              value={fontSize}
              onChange={(e) => setFontSize(parseInt(e.target.value))}
              disabled={isLoading}
            />
          </label>
        </div>

        <div className="playback-controls">
          {isReading ? (
            <button
              className="stop-button"
              onClick={stopReading}
              disabled={isLoading || phrases.length === 0}
            >
              停止
            </button>
          ) : (
            <button
              className="start-button"
              onClick={startReading}
              disabled={isLoading || phrases.length === 0}
            >
              読書開始
            </button>
          )}

          <button
            onClick={prevPhrase}
            disabled={
              isLoading ||
              isReading ||
              currentPhraseIndex === 0 ||
              phrases.length === 0
            }
          >
            前へ
          </button>

          <button
            onClick={nextPhrase}
            disabled={
              isLoading ||
              isReading ||
              currentPhraseIndex === phrases.length - 1 ||
              phrases.length === 0
            }
          >
            次へ
          </button>
        </div>
      </div>

      {/* エラーメッセージ */}
      {error && <div className="error">{error}</div>}

      {/* ローディング表示 */}
      {isLoading ? (
        <div className="loading">テキストを処理中...</div>
      ) : (
        <>
          {/* 文節表示エリア */}
          <div
            className="phrase-display"
            style={{
              fontSize: `${fontSize}px`,
            }}
          >
            {phrases.length > 0 ? (
              isReading || currentPhraseIndex > 0 ? (
                <div className="current-phrase">
                  {phrases[currentPhraseIndex]}
                </div>
              ) : (
                <div className="start-message">
                  「読書開始」ボタンを押してください
                </div>
              )
            ) : (
              <div className="no-text">テキストを入力してください</div>
            )}
          </div>

          {/* 進捗バー */}
          {phrases.length > 0 && (
            <div className="progress">
              <div
                className="progress-bar"
                style={{
                  width: `${
                    ((currentPhraseIndex + 1) / phrases.length) * 100
                  }%`,
                }}
              ></div>
              <div className="progress-text">
                {currentPhraseIndex + 1} / {phrases.length}
              </div>
            </div>
          )}
        </>
      )}
    </div>
  );
}

export default BunpitsuReader;

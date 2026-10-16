// 瀏覽器內建 speechSynthesis 朗讀(vocab 單字發音、farm 語音導覽共用):不打外部 TTS、不放音檔。
// 語音品質完全看使用者裝置裝了哪些 voice,所以只當「加分功能」——
// 不支援就把按鈕藏起來,不要留一顆按了沒反應的鈕。

/** 有沒有可用的語音引擎(必須在 mount 後才問,SSR 沒有 window) */
export function canSpeak(): boolean {
    return typeof window !== "undefined" && "speechSynthesis" in window;
}

/** `lang` 是 BCP 47(`ja-JP` / `zh-TW` / `en-US`)。完全相符優先,其次同語言前綴 */
function pickVoice(lang: string): SpeechSynthesisVoice | undefined {
    const prefix = lang.split("-")[0];
    const voices = window.speechSynthesis.getVoices();
    // getVoices() 在部分瀏覽器首次呼叫是空的(要等 voiceschanged);
    // 空的就不指定 voice,交給 utterance.lang 讓引擎自己挑。
    return (
        voices.find(v => v.lang.replace("_", "-") === lang) ??
        voices.find(v => v.lang.replace("_", "-").startsWith(prefix))
    );
}

/** 朗讀一段文字;連點只留最後一次,不疊音 */
export function speakText(text: string, lang: string, rate = 1) {
    const content = text.trim();
    if (!canSpeak() || !content) return;
    const synth = window.speechSynthesis;
    synth.cancel();
    const utterance = new SpeechSynthesisUtterance(content);
    utterance.lang = lang;
    utterance.rate = rate;
    const voice = pickVoice(lang);
    if (voice) utterance.voice = voice;
    synth.speak(utterance);
}

/** 停止朗讀(離開頁面 / 關閉語音時) */
export function stopSpeaking() {
    if (canSpeak()) window.speechSynthesis.cancel();
}

let unlocked = false;
/** iOS Safari 要在使用者手勢內念過一次才允許之後自動念;接在第一次點擊上 */
export function unlockSpeech() {
    if (unlocked || !canSpeak()) return;
    unlocked = true;
    const u = new SpeechSynthesisUtterance(" ");
    u.volume = 0;
    window.speechSynthesis.speak(u);
}

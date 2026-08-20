// 單字發音:走瀏覽器內建 speechSynthesis,不打外部 TTS、不放音檔。
// 語音品質完全看使用者裝置裝了哪些 voice,所以只當「加分功能」——
// 不支援就把按鈕藏起來,不要留一顆按了沒反應的鈕。

import type { VocabLanguage } from "@/types";

const LANG_TAG: Record<VocabLanguage, string> = { en: "en-US", ja: "ja-JP" };

/** 有沒有可用的語音引擎(必須在 mount 後才問,SSR 沒有 window) */
export function canSpeak(): boolean {
    return typeof window !== "undefined" && "speechSynthesis" in window;
}

function pickVoice(language: VocabLanguage): SpeechSynthesisVoice | undefined {
    const tag = LANG_TAG[language];
    const voices = window.speechSynthesis.getVoices();
    // getVoices() 在部分瀏覽器首次呼叫是空的(要等 voiceschanged);
    // 空的就不指定 voice,交給 utterance.lang 讓引擎自己挑。
    return (
        voices.find(v => v.lang.replace("_", "-") === tag) ??
        voices.find(v => v.lang.replace("_", "-").startsWith(language))
    );
}

/** 唸一個單字/讀音;連點只留最後一次,不疊音 */
export function speak(text: string, language: VocabLanguage) {
    const word = text.trim();
    if (!canSpeak() || !word) return;
    const synth = window.speechSynthesis;
    synth.cancel();
    const utterance = new SpeechSynthesisUtterance(word);
    utterance.lang = LANG_TAG[language];
    utterance.rate = language === "ja" ? 0.9 : 0.95; // 預設速度對單字太快
    const voice = pickVoice(language);
    if (voice) utterance.voice = voice;
    synth.speak(utterance);
}

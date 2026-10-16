// 單字發音:朗讀核心在 @/libs/speech(與 farm 語音導覽共用),此檔只留單字的語系與語速。

import type { VocabLanguage } from "@/types";
import { speakText } from "@/libs/speech";

export { canSpeak } from "@/libs/speech";

const LANG_TAG: Record<VocabLanguage, string> = { en: "en-US", ja: "ja-JP" };

/** 唸一個單字/讀音;連點只留最後一次,不疊音 */
export function speak(text: string, language: VocabLanguage) {
    speakText(text, LANG_TAG[language], language === "ja" ? 0.9 : 0.95); // 預設速度對單字太快
}

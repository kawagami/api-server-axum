//! 日文讀音正規化:玩家輸入(羅馬字 / 平假名 / 片假名)統一轉平假名後嚴格比對。
//! 規則走嚴格路線——長音省略(tokyo ≠ とうきょう)不容錯,
//! 前端即時把輸入轉成假名顯示,玩家看得到自己打了什麼。

use unicode_normalization::UnicodeNormalization;
use wana_kana::{ConvertJapanese, Options};

/// NFKC → 去空白 → 小寫 → 轉平假名(羅馬字/片假名 → 平假名,平假名原樣)。
/// 長音符「ー」保留原樣不展開:預設會把片假名的ー展開成母音(コーヒー→こうひい),
/// 但平假名輸入的ー會原樣通過,兩邊不對稱 → 一律保留,比對才對稱。
pub fn normalize_reading(input: &str) -> String {
    let cleaned: String = input
        .nfkc()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_lowercase();
    cleaned.to_hiragana_with_opt(Options {
        keep_prolonged_sound_mark: true,
        ..Options::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn romaji_converts_to_hiragana() {
        assert_eq!(normalize_reading("taberu"), "たべる");
        // 大寫在 WanaKana 慣例是片假名,先小寫避免
        assert_eq!(normalize_reading("TABERU"), "たべる");
        assert_eq!(normalize_reading(" taberu "), "たべる");
    }

    #[test]
    fn kana_input_passes_through_as_hiragana() {
        assert_eq!(normalize_reading("たべる"), "たべる");
        // 片假名(外來語詞或 IME 誤切)一律折回平假名
        assert_eq!(normalize_reading("タベル"), "たべる");
        // 長音符「ー」保留原樣:片假名/平假名輸入正規化後必須一致
        assert_eq!(normalize_reading("コーヒー"), "こーひー");
        assert_eq!(normalize_reading("こーひー"), normalize_reading("コーヒー"));
    }

    #[test]
    fn hepburn_and_kunrei_both_accepted() {
        assert_eq!(normalize_reading("shi"), normalize_reading("si"));
        assert_eq!(normalize_reading("tsu"), normalize_reading("tu"));
        assert_eq!(normalize_reading("fu"), normalize_reading("hu"));
        assert_eq!(normalize_reading("ji"), normalize_reading("zi"));
    }

    #[test]
    fn sokuon_and_n() {
        assert_eq!(normalize_reading("kitte"), "きって");
        assert_eq!(normalize_reading("shinbun"), "しんぶん");
        assert_eq!(normalize_reading("sannin"), "さんにん");
    }

    #[test]
    fn long_vowel_omission_is_not_tolerated() {
        // 嚴格路線:省略長音就是不對
        assert_eq!(normalize_reading("toukyou"), "とうきょう");
        assert_ne!(normalize_reading("tokyo"), "とうきょう");
    }

    #[test]
    fn fullwidth_input_is_normalized() {
        // NFKC 把全形英數折回半形
        assert_eq!(normalize_reading("ｔａｂｅｒｕ"), "たべる");
        // 全形空白也去掉
        assert_eq!(normalize_reading("たべ\u{3000}る"), "たべる");
    }

    #[test]
    fn multiple_readings_normalize_independently() {
        // 明日:あした / あす 各自正規化後可分別比對
        assert_eq!(normalize_reading("ashita"), "あした");
        assert_eq!(normalize_reading("asu"), "あす");
    }
}

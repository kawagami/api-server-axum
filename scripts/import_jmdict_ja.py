#!/usr/bin/env python3
"""日文題庫匯入工具:JMdict × JLPT 字表 → 翻譯 TSV → 驗證 → migration。

與英文版(add_vocab_words.py)不同,讀音/詞性/表記取 JMdict 權威值,
LLM 只負責把英文 gloss 翻成中文釋義;規格見 docs/2026-07-10-japanese-vocab-mvp-spec.md。

流程:
  1. 抽取(配對 JMdict、產出待翻譯 TSV;meaning_zh 欄留空):
       python3 scripts/import_jmdict_ja.py extract \
         --jmdict JMdict_e --jlpt n5.csv=1 --jlpt n4.csv=2 \
         [--limit 50] --out-tsv ja_batch.tsv
  2. LLM 將 TSV 的 meaning_zh 欄填滿(≤20 字,參考 gloss_en 與 pos)
  3. 組裝(機械驗證 + 產 migration;--review-tsv 給強 model 語意審核):
       python3 scripts/import_jmdict_ja.py assemble \
         --filled-tsv ja_batch.filled.tsv --name vocab_seed_ja_pilot

資料來源與授權:
  - JMdict(EDRDG,CC BY-SA 4.0)——站上需標註出處
  - JLPT 分級為社群整理(tanos.co.uk 系),是估計值非官方
"""
from __future__ import annotations

import argparse
import csv
import datetime
import re
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
MIG_DIR = REPO_ROOT / "backend" / "migrations"

ROWS_PER_STMT = 50
MEANING_MAX_CHARS = 20
# 假名 + 長音符(讀音欄位允許的字元);外來語保留片假名顯示,
# 比對端(後端 normalize_reading)自會折平假名
READING_RE = re.compile(r"^[ぁ-ゖァ-ヶー]+$")
# 既有 migration 中的日文列:('ja', 'word', 'reading', ...)
JA_ROW_RE = re.compile(r"\(\s*'ja'\s*,\s*'((?:[^']|'')+)'\s*,\s*'((?:[^']|'')+)'")

PRIORITY_TAGS = {"news1", "ichi1", "spec1", "spec2", "gai1"}
# 排除的讀音注記(罕用/過時/僅供搜尋的假名形,不收進 accepted_readings)
BAD_RE_INF = ("irregular", "out-dated", "outdated", "search-only")
USUALLY_KANA = "usually written using kana alone"

ALLOWED_POS_ZH = {
    "一段動詞", "五段動詞", "する動詞", "カ変動詞", "い形容詞", "な形容詞",
    "名詞", "代名詞", "連體詞", "副詞", "感動詞", "接續詞", "助詞", "助動詞",
    "量詞", "數詞", "慣用語", "接頭詞", "接尾詞", "其他",
}


def kata_to_hira(s: str) -> str:
    """片假名折回平假名;長音符「ー」保留(與後端 normalize_reading 一致)。"""
    return "".join(
        chr(ord(c) - 0x60) if 0x30A1 <= ord(c) <= 0x30F6 else c for c in s
    )


def has_kanji(s: str) -> bool:
    return any("一" <= c <= "鿿" or c in "々〆" for c in s)


def classify_pos(text: str) -> str | None:
    """單一 pos 描述 → 中文詞性;認不得回 None。
    注意順序與精確性:「nouns which may take the genitive case particle 'no'」
    含 "particle" 子字串,不能用鬆散的關鍵字先搶。"""
    if "Ichidan verb" in text:
        return "一段動詞"
    if "Godan verb" in text:
        return "五段動詞"
    if "suru verb" in text or "takes the aux. verb suru" in text:
        return "する動詞"
    if "Kuru verb" in text:
        return "カ変動詞"
    if "(keiyoushi)" in text:
        return "い形容詞"
    if "(keiyodoshi)" in text:
        return "な形容詞"
    if "pre-noun adjectival" in text:
        return "連體詞"
    if text.startswith("noun"):
        return "名詞"
    if text.startswith("adverb"):
        return "副詞"
    if text.startswith("pronoun"):
        return "代名詞"
    if text.startswith("interjection"):
        return "感動詞"
    if text.startswith("conjunction"):
        return "接續詞"
    if text.startswith("particle"):
        return "助詞"
    if text.startswith("auxiliary"):
        return "助動詞"
    if text.startswith("counter"):
        return "量詞"
    if text.startswith("numeric"):
        return "數詞"
    if text.startswith("expression"):
        return "慣用語"
    if text.startswith("prefix"):
        return "接頭詞"
    if text.startswith("suffix"):
        return "接尾詞"
    return None


def map_pos(pos_texts: list[str]) -> str:
    """逐個 pos 依序分類,先分出來的贏(JMdict 把主要詞性放前面)。
    例外:する動詞優先——勉強/結婚這類 JMdict 標 [noun, vs],
    對學習者「する動詞」比「名詞」有用。"""
    joined = " / ".join(pos_texts)
    if "takes the aux. verb suru" in joined or "suru verb" in joined:
        return "する動詞"
    for text in pos_texts:
        zh = classify_pos(text)
        if zh:
            return zh
    return "其他"


STOPWORDS = {"to", "a", "an", "the", "of", "in", "on", "at", "for", "or",
             "and", "etc", "eg", "e.g", "sth", "one's", "be", "not"}


def gloss_tokens(text: str) -> set[str]:
    return {
        t for t in re.findall(r"[a-z]+", text.lower()) if t not in STOPWORDS
    }


def gloss_overlap(csv_meaning: str, gloss: str) -> int:
    """CSV 釋義與 JMdict gloss 的字詞重疊分(詞條/sense 消歧用)。
    字表把主要意思放最前面,首段(第一個逗號/分號前)命中加倍計分,
    避免「あの = that over there; um...」被 um 平手拉去感動詞詞條。"""
    g = gloss_tokens(gloss)
    primary = gloss_tokens(re.split(r"[,;]", csv_meaning, 1)[0])
    return 2 * len(primary & g) + len(gloss_tokens(csv_meaning) & g)


# ---------- extract ----------

def parse_jmdict(path: Path):
    """單趟掃描 JMdict,建 keb / reb 兩個索引(entry 只留需要的欄位)。"""
    by_keb: dict[str, list[dict]] = {}
    by_reb: dict[str, list[dict]] = {}
    for _, el in ET.iterparse(str(path), events=("end",)):
        if el.tag != "entry":
            continue
        kebs = [k.findtext("keb") for k in el.findall("k_ele")]
        pri = set()
        for tag in el.iter("ke_pri"):
            pri.add(tag.text)
        for tag in el.iter("re_pri"):
            pri.add(tag.text)
        rebs = []
        for r in el.findall("r_ele"):
            infs = [x.text or "" for x in r.findall("re_inf")]
            rebs.append({
                "reb": r.findtext("reb"),
                "restr": [x.text for x in r.findall("re_restr")],
                "bad": any(b in inf for inf in infs for b in BAD_RE_INF),
            })
        # sense 的 pos 沒寫時繼承前一個 sense(JMdict 慣例)
        senses, last_pos = [], []
        for s in el.findall("sense"):
            pos_texts = [p.text for p in s.findall("pos") if p.text] or last_pos
            last_pos = pos_texts
            senses.append({
                "pos": pos_texts,
                "gloss": "; ".join(g.text for g in s.findall("gloss") if g.text),
                "misc": [m.text or "" for m in s.findall("misc")],
            })
        entry = {
            "kebs": kebs,
            "rebs": rebs,
            "senses": senses,
            "priority": bool(pri & PRIORITY_TAGS),
        }
        for keb in kebs:
            by_keb.setdefault(keb, []).append(entry)
        for r in rebs:
            by_reb.setdefault(r["reb"], []).append(entry)
        el.clear()
    return by_keb, by_reb


def readings_for_surface(entry: dict, surface: str) -> list[str]:
    """該表記適用的全部讀音(尊重 re_restr、排除罕用/過時假名形),原樣保留、去重。"""
    out: list[str] = []
    seen_h: set[str] = set()
    for r in entry["rebs"]:
        if r["bad"]:
            continue
        if r["restr"] and surface not in r["restr"]:
            continue
        h = kata_to_hira(r["reb"])
        if h not in seen_h:
            seen_h.add(h)
            out.append(r["reb"])
    return out


def match_entry(expr: str, reading_h: str, csv_meaning: str, by_keb, by_reb):
    """以 (表記, 讀音) 配對 JMdict 詞條與 sense。
    多候選時用 CSV 英文釋義的字詞重疊消歧(避免 いくら 配到鮭魚卵 イクラ),
    再看常用度標記;回 (entry, sense) 或 None。"""
    def entry_readings_h(e, surface):
        return [kata_to_hira(r) for r in readings_for_surface(e, surface)]

    if has_kanji(expr):
        candidates = [
            e for e in by_keb.get(expr, [])
            if reading_h in entry_readings_h(e, expr)
        ]
    else:
        # 假名詞:JLPT 字表以假名出現(即使 JMdict 有罕用漢字表記,如 有る)
        candidates = [
            e for e in by_reb.get(expr, [])
            if reading_h in [kata_to_hira(r["reb"]) for r in e["rebs"]]
        ]
    if not candidates:
        return None

    def best_sense(e):
        scored = [
            (gloss_overlap(csv_meaning, s["gloss"]), -i, s)
            for i, s in enumerate(e["senses"])
        ]
        return max(scored) if scored else (0, 0, None)

    ranked = []
    for e in candidates:
        overlap, neg_i, sense = best_sense(e)
        ranked.append((overlap, e["priority"], neg_i, id(e), e, sense))
    ranked.sort(reverse=True)
    _, _, _, _, entry, sense = ranked[0]
    if sense is None:
        return None
    return entry, sense


def cmd_extract(args) -> int:
    jlpt_files: list[tuple[Path, int]] = []
    for spec in args.jlpt:
        path, _, diff = spec.partition("=")
        if not diff.isdigit() or not 1 <= int(diff) <= 5:
            sys.exit(f"--jlpt 格式須為 檔案=難度(1-5):{spec}")
        jlpt_files.append((Path(path), int(diff)))

    print("解析 JMdict…", file=sys.stderr)
    by_keb, by_reb = parse_jmdict(Path(args.jmdict))
    print(f"JMdict 索引:{len(by_keb)} 表記 / {len(by_reb)} 讀音", file=sys.stderr)

    rows, unmatched, seen_keys = [], [], set()
    for csv_path, difficulty in jlpt_files:
        with open(csv_path, newline="", encoding="utf-8") as f:
            for line in csv.DictReader(f):
                # 字表偶有「足; 脚」「いく; ゆく」多形式列:各取第一形,
                # 其餘讀音反正會從 JMdict 的 accepted_readings 補回。
                # 另清掉字表慣用注記:量詞/接尾的「～」、する動詞的「(する)」
                expr = line["expression"].split(";")[0].replace("～", "").strip()
                reading_raw = line["reading"].split(";")[0].replace("～", "")
                reading_raw = re.sub(r"\s*[((]する[))]\s*$", "", reading_raw).strip()
                csv_meaning = line.get("meaning", "")
                # 字表雜訊:reading 欄反而是漢字(あいさつする(挨拶))→ 兩欄對調
                if has_kanji(reading_raw) and not has_kanji(expr):
                    expr, reading_raw = reading_raw, expr
                # 「運動(うんどうする)」型:讀音尾帶する但表記沒有 → 去掉
                if reading_raw.endswith("する") and not expr.endswith("する"):
                    reading_raw = reading_raw[:-2]
                reading_h = kata_to_hira(reading_raw)
                if not expr or not READING_RE.match(reading_raw):
                    unmatched.append((expr, line["reading"], "讀音非假名"))
                    continue
                matched = match_entry(expr, reading_h, csv_meaning, by_keb, by_reb)
                # 「コピーする」型:去尾する當名詞/する動詞再試一次
                if matched is None and expr.endswith("する") and reading_h.endswith("する"):
                    expr, reading_raw = expr[:-2], reading_raw[:-2]
                    reading_h = reading_h[:-2]
                    matched = match_entry(expr, reading_h, csv_meaning, by_keb, by_reb)
                if matched is None:
                    unmatched.append((expr, line["reading"], "JMdict 無配對"))
                    continue
                entry, sense = matched
                # 「通常寫假名」的詞(有る、幾ら)改用假名當表記,
                # N5/N4 教材慣例如此;同讀音的多漢字形會在此收斂去重
                word = expr
                if has_kanji(expr) and any(USUALLY_KANA in m for m in sense["misc"]):
                    word = reading_raw
                key = (word, reading_h)
                if key in seen_keys:
                    continue
                seen_keys.add(key)
                if has_kanji(word):
                    accepted = readings_for_surface(entry, expr)
                else:
                    accepted = [r["reb"] for r in entry["rebs"] if not r["bad"]]
                accepted = [r for r in accepted if READING_RE.match(r)]
                # 去重以折平假名後為準(イクラ/いくら 同音只留一個)
                dedup: dict[str, str] = {}
                for r in [reading_raw] + accepted:
                    dedup.setdefault(kata_to_hira(r), r)
                rows.append({
                    "word": word,
                    "reading": reading_raw,
                    "accepted_readings": "|".join(dedup.values()),
                    "pos": map_pos(sense["pos"]),
                    "difficulty": difficulty,
                    "gloss_en": "; ".join(sense["gloss"].split("; ")[:3])
                        or csv_meaning,
                    "meaning_zh": "",
                })
                if args.limit and len(rows) >= args.limit:
                    break
        if args.limit and len(rows) >= args.limit:
            break

    with open(args.out_tsv, "w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0].keys()), delimiter="\t")
        w.writeheader()
        w.writerows(rows)

    pos_other = [r["word"] for r in rows if r["pos"] == "其他"]
    print(f"配對成功 {len(rows)} 字 → {args.out_tsv}", file=sys.stderr)
    if pos_other:
        print(f"詞性無法映射({len(pos_other)}):{' '.join(pos_other[:10])}", file=sys.stderr)
    if unmatched:
        print(f"未配對 {len(unmatched)}:", file=sys.stderr)
        for expr, reading, why in unmatched[:20]:
            print(f"  {expr}({reading}):{why}", file=sys.stderr)
    return 0


# ---------- assemble ----------

def existing_ja_keys() -> set[tuple[str, str]]:
    keys: set[tuple[str, str]] = set()
    for path in MIG_DIR.glob("*.up.sql"):
        text = path.read_text()
        if "INSERT INTO words" not in text:
            continue
        for w, r in JA_ROW_RE.findall(text):
            keys.add((w.replace("''", "'"), r.replace("''", "'")))
    return keys


def esc(s: str) -> str:
    return s.replace("'", "''")


def cmd_assemble(args) -> int:
    with open(args.filled_tsv, newline="", encoding="utf-8") as f:
        rows = list(csv.DictReader(f, delimiter="\t"))

    seen = existing_ja_keys()
    ok, errors = [], []
    batch_keys: set[tuple[str, str]] = set()
    for i, r in enumerate(rows, 2):  # 行號含表頭
        word, reading = r["word"].strip(), r["reading"].strip()
        meaning = r["meaning_zh"].strip()
        accepted = [a for a in r["accepted_readings"].split("|") if a]
        problems = []
        if not word:
            problems.append("word 空")
        if not READING_RE.match(reading):
            problems.append("reading 非假名")
        if any(not READING_RE.match(a) for a in accepted):
            problems.append("accepted_readings 含非假名")
        if reading not in accepted:
            problems.append("reading 不在 accepted_readings 內")
        if not meaning:
            problems.append("meaning_zh 未填")
        elif len(meaning) > MEANING_MAX_CHARS:
            problems.append(f"釋義超過 {MEANING_MAX_CHARS} 字")
        if r["pos"] not in ALLOWED_POS_ZH:
            problems.append(f"詞性不在白名單:{r['pos']}")
        if not r["difficulty"].isdigit() or not 1 <= int(r["difficulty"]) <= 5:
            problems.append("難度超界")
        if (word, reading) in seen or (word, reading) in batch_keys:
            problems.append("重複(已在 migration 或本批)")
        if problems:
            errors.append(f"  第 {i} 行 {word}({reading}):{'、'.join(problems)}")
            continue
        batch_keys.add((word, reading))
        ok.append(r)

    print(f"驗證通過 {len(ok)} / {len(rows)} 字")
    if errors:
        print("擋下:")
        print("\n".join(errors))
    if not ok:
        sys.exit("沒有可用資料")

    if args.review_tsv:
        with open(args.review_tsv, "w", newline="", encoding="utf-8") as f:
            w = csv.writer(f, delimiter="\t")
            w.writerow(["word", "reading", "pos", "meaning_zh", "gloss_en", "difficulty"])
            for r in ok:
                w.writerow([r["word"], r["reading"], r["pos"], r["meaning_zh"],
                            r["gloss_en"], r["difficulty"]])
        print(f"語意審核清單 → {args.review_tsv}")

    if args.dry_run:
        return 0
    if not args.name:
        sys.exit("--name 必填(或用 --dry-run)")

    ts = datetime.datetime.now().strftime("%Y%m%d%H%M%S")
    up_path = MIG_DIR / f"{ts}_{args.name}.up.sql"
    down_path = MIG_DIR / f"{ts}_{args.name}.down.sql"

    stmts = []
    for start in range(0, len(ok), ROWS_PER_STMT):
        values = []
        for r in ok[start:start + ROWS_PER_STMT]:
            arr = ", ".join(f"'{esc(a)}'" for a in dict.fromkeys(
                x for x in r["accepted_readings"].split("|") if x))
            values.append(
                f"('ja', '{esc(r['word'])}', '{esc(r['reading'])}', ARRAY[{arr}], "
                f"'{esc(r['pos'])}', '{esc(r['meaning_zh'].strip())}', '', {int(r['difficulty'])})"
            )
        stmts.append(
            "INSERT INTO words (language, word, reading, accepted_readings, "
            "part_of_speech, meaning_zh, example_sentence, difficulty)\nVALUES\n    "
            + ",\n    ".join(values)
            + "\nON CONFLICT ON CONSTRAINT words_lang_word_reading_key DO NOTHING;"
        )

    header = (
        f"-- 日文題庫:{args.name}({len(ok)} 字)\n"
        "-- 讀音/詞性取自 JMdict(EDRDG,CC BY-SA 4.0);JLPT 分級為社群估計值\n"
        "-- 產生流程見 scripts/import_jmdict_ja.py\n\n"
    )
    up_path.write_text(header + "\n\n".join(stmts) + "\n")

    keys = ",\n    ".join(
        f"('{esc(r['word'])}', '{esc(r['reading'])}')" for r in ok
    )
    down_path.write_text(
        f"DELETE FROM words WHERE language = 'ja' AND (word, reading) IN (\n    {keys}\n);\n"
    )
    print(f"migration → {up_path.name} / {down_path.name}")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    sub = ap.add_subparsers(dest="cmd", required=True)

    ex = sub.add_parser("extract", help="JMdict × JLPT 字表 → 待翻譯 TSV")
    ex.add_argument("--jmdict", required=True, help="JMdict_e XML 路徑")
    ex.add_argument("--jlpt", action="append", required=True,
                    help="JLPT 字表 CSV=難度,如 n5.csv=1(可多次)")
    ex.add_argument("--limit", type=int, help="最多抽幾字(先導批用)")
    ex.add_argument("--out-tsv", required=True, help="輸出 TSV 路徑")
    ex.set_defaults(fn=cmd_extract)

    asm = sub.add_parser("assemble", help="已翻譯 TSV → 驗證 + migration")
    asm.add_argument("--filled-tsv", required=True, help="meaning_zh 已填的 TSV")
    asm.add_argument("--name", help="migration 名稱,如 vocab_seed_ja_pilot")
    asm.add_argument("--review-tsv", type=Path, help="輸出語意審核清單")
    asm.add_argument("--dry-run", action="store_true", help="只驗證,不寫 migration")
    asm.set_defaults(fn=cmd_assemble)

    args = ap.parse_args()
    return args.fn(args)


if __name__ == "__main__":
    sys.exit(main())

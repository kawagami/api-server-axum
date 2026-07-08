#!/usr/bin/env python3
"""單字題庫擴充工具:驗證生成批次、去重、輸出 migration。

流程(完整規則見 docs/2026-07-08-vocab-word-seeding.md):
  1. 取得現有字清單(餵給生成 model 當排除清單):
       python3 scripts/add_vocab_words.py --print-existing
  2. 任意 model 依規則文件生成批次 SQL 後,驗證 + 組裝 migration:
       python3 scripts/add_vocab_words.py batch1.sql batch2.sql --name vocab_seed_batch3
  3. 產出語意審核清單(給強 model 核對釋義):
       加 --review-tsv review.tsv

機械驗證只擋格式問題(重複/例句不含原形/難度範圍/SQL 跳脫);
釋義是否翻對要靠步驟 3 的語意審核,腳本驗不了。
"""
from __future__ import annotations

import argparse
import datetime
import re
import sys
from collections import Counter
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
MIG_DIR = REPO_ROOT / "backend" / "migrations"

ALLOWED_POS = {"n.", "v.", "adj.", "adv.", "prep.", "conj.", "pron.", "int."}
MEANING_MAX_CHARS = 20

# 一列資料:('word', 'pos', 'meaning', 'example', d)
ROW_RE = re.compile(
    r"\(\s*'([a-z]+)'\s*,\s*'((?:[^']|'')*)'\s*,\s*'((?:[^']|'')*)'\s*,\s*'((?:[^']|'')*)'\s*,\s*(\d)\s*\)"
)


def existing_words() -> set[str]:
    """從所有含 words INSERT 的 migration 蒐集既有單字。"""
    words: set[str] = set()
    for path in MIG_DIR.glob("*.up.sql"):
        text = path.read_text()
        if "INSERT INTO words" not in text:
            continue
        words.update(w for w, *_ in ROW_RE.findall(text))
    return words


def example_has_base_form(word: str, example: str) -> bool:
    plain = example.replace("''", "'")
    pattern = rf"\b({re.escape(word)}|{re.escape(word.capitalize())})\b"
    return re.search(pattern, plain) is not None


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("batches", nargs="*", help="生成批次 SQL 檔(可多個)")
    ap.add_argument("--name", help="migration 名稱(不含時間戳),如 vocab_seed_batch3")
    ap.add_argument("--print-existing", action="store_true", help="只印出現有單字清單(空白分隔)後結束")
    ap.add_argument("--review-tsv", type=Path, help="輸出語意審核清單(word\\tpos\\tmeaning\\tdifficulty)")
    ap.add_argument("--dry-run", action="store_true", help="只驗證統計,不寫 migration")
    args = ap.parse_args()

    seen = existing_words()

    if args.print_existing:
        print(" ".join(sorted(seen)))
        return 0

    if not args.batches:
        ap.error("需要至少一個批次 SQL 檔(或 --print-existing)")
    if not args.dry_run and not args.name:
        ap.error("--name 必填(或用 --dry-run)")

    print(f"existing words in migrations: {len(seen)}")

    rows = []
    dropped_dup, dropped_bad_example, dropped_bad_difficulty = [], [], []
    warnings = []

    for batch in args.batches:
        text = Path(batch).read_text()
        found = ROW_RE.findall(text)
        print(f"{batch}: parsed {len(found)} rows")
        for word, pos, meaning, example, diff in found:
            if word in seen:
                dropped_dup.append(word)
                continue
            if not example_has_base_form(word, example):
                dropped_bad_example.append(word)
                continue
            if not 1 <= int(diff) <= 5:
                dropped_bad_difficulty.append(word)
                continue
            # 風格檢查:只警告不丟棄,人工斟酌
            if pos not in ALLOWED_POS:
                warnings.append(f"{word}: 非常見詞性標記 '{pos}'")
            if len(meaning.replace("''", "'")) > MEANING_MAX_CHARS:
                warnings.append(f"{word}: 釋義過長({len(meaning)} 字)「{meaning}」")
            if re.search(r"[,:;!?]", meaning):
                warnings.append(f"{word}: 釋義含半形標點「{meaning}」")
            seen.add(word)
            rows.append((word, pos, meaning, example, int(diff)))

    print(f"\nkept: {len(rows)}")
    print(f"dropped duplicates ({len(dropped_dup)}): {dropped_dup}")
    print(f"dropped bad example ({len(dropped_bad_example)}): {dropped_bad_example}")
    print(f"dropped bad difficulty ({len(dropped_bad_difficulty)}): {dropped_bad_difficulty}")
    dist = Counter(d for *_, d in rows)
    print("difficulty distribution:", dict(sorted(dist.items())))
    if warnings:
        print(f"\nstyle warnings ({len(warnings)}):")
        for w in warnings:
            print(f"  - {w}")

    if not rows:
        print("nothing to write")
        return 1

    if args.review_tsv:
        with open(args.review_tsv, "w") as f:
            for word, pos, meaning, _, diff in rows:
                f.write(f"{word}\t{pos}\t{meaning}\t{diff}\n")
        print(f"\nwrote review list: {args.review_tsv}")

    if args.dry_run:
        return 0

    ts = datetime.datetime.now().strftime("%Y%m%d%H%M%S")
    up_path = MIG_DIR / f"{ts}_{args.name}.up.sql"
    down_path = MIG_DIR / f"{ts}_{args.name}.down.sql"

    lines = [f"-- 英文單字題庫擴充:+{len(rows)} 字(scripts/add_vocab_words.py 產生)", ""]
    for i in range(0, len(rows), 50):
        chunk = rows[i:i + 50]
        lines.append("INSERT INTO words (word, part_of_speech, meaning_zh, example_sentence, difficulty) VALUES")
        lines.append(",\n".join(f"('{w}', '{p}', '{m}', '{e}', {d})" for w, p, m, e, d in chunk))
        lines.append("ON CONFLICT (word) DO NOTHING;")
        lines.append("")
    up_path.write_text("\n".join(lines))

    words_list = ", ".join(f"'{w}'" for w, *_ in rows)
    down_path.write_text(f"DELETE FROM words WHERE word IN ({words_list});\n")

    print(f"\nwrote {up_path.name} / {down_path.name}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

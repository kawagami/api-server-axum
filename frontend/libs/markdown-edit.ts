// Pure markdown-textarea edit transforms.
// Each takes the current value + selection range and returns the next value
// plus where the caret/selection should land. Zero DOM access → unit-testable.

export const INDENT = '\t';

export interface EditResult {
    value: string;
    start: number;
    end: number;
}

export function isUrl(text: string): boolean {
    return /^https?:\/\/\S+$/.test(text);
}

/** Insert text at the cursor, replacing any selection. */
export function insert(value: string, start: number, end: number, text: string): EditResult {
    const caret = start + text.length;
    return { value: value.slice(0, start) + text + value.slice(end), start: caret, end: caret };
}

/** Tab: indent the whole selected block (multi-line) or insert one indent. */
export function indent(value: string, start: number, end: number): EditResult {
    const multiLine = value.slice(start, end).includes('\n');
    if (!multiLine) return insert(value, start, end, INDENT);

    const lineStart = value.lastIndexOf('\n', start - 1) + 1;
    const before = value.slice(0, lineStart);
    const block = value.slice(lineStart, end);
    const after = value.slice(end);
    const lines = block.split('\n');
    const newBlock = lines.map(l => INDENT + l).join('\n');
    return {
        value: before + newBlock + after,
        start: start + INDENT.length,
        end: end + INDENT.length * lines.length,
    };
}

/** Shift+Tab: strip one indent (up to 2 spaces or a tab) from each line in range. */
export function unindent(value: string, start: number, end: number): EditResult {
    const lineStart = value.lastIndexOf('\n', start - 1) + 1;
    const before = value.slice(0, lineStart);
    const block = value.slice(lineStart, end);
    const after = value.slice(end);
    let removedFirst = 0;
    let removedTotal = 0;
    const newBlock = block.split('\n').map((line, i) => {
        const m = line.match(/^( {1,2}|\t)/);
        const cut = m ? m[0].length : 0;
        if (i === 0) removedFirst = cut;
        removedTotal += cut;
        return line.slice(cut);
    }).join('\n');
    return {
        value: before + newBlock + after,
        start: Math.max(lineStart, start - removedFirst),
        end: end - removedTotal,
    };
}

/** Enter on a `- `/`* ` list line: continue the list, or exit on an empty item.
 *  Returns null when the cursor isn't on a list line (let the browser handle it). */
export function continueList(value: string, start: number, end: number): EditResult | null {
    const lineStart = value.lastIndexOf('\n', start - 1) + 1;
    const currentLine = value.slice(lineStart, start);
    const match = currentLine.match(/^(\s*[-*]\s)/);
    if (!match) return null;

    const prefix = match[1];
    const lineContent = currentLine.slice(prefix.length);
    if (!lineContent) {
        // empty list item → drop the marker, exit the list
        const caret = lineStart + 1;
        return { value: value.slice(0, lineStart) + '\n' + value.slice(end), start: caret, end: caret };
    }
    const insertion = '\n' + prefix;
    const caret = start + insertion.length;
    return { value: value.slice(0, start) + insertion + value.slice(end), start: caret, end: caret };
}

/** Wrap the current selection as a markdown link `[selected](url)`.
 *  Returns null when there's no selection. */
export function wrapLink(value: string, start: number, end: number, url: string): EditResult | null {
    if (start === end) return null;
    const link = `[${value.slice(start, end)}](${url})`;
    const caret = start + link.length;
    return { value: value.slice(0, start) + link + value.slice(end), start: caret, end: caret };
}

/** Wrap the selection with `before`/`after` markers (e.g. `**bold**`), dropping
 *  in `placeholder` when nothing is selected. Caret selects the inner text. */
export function surround(
    value: string, start: number, end: number,
    before: string, after: string = before, placeholder = '',
): EditResult {
    const inner = value.slice(start, end) || placeholder;
    const text = before + inner + after;
    return {
        value: value.slice(0, start) + text + value.slice(end),
        start: start + before.length,
        end: start + before.length + inner.length,
    };
}

/** Prepend `marker` to the start of every line the selection touches
 *  (heading/quote/list). Empty selection prefixes just the caret line. */
export function prefixLines(value: string, start: number, end: number, marker: string): EditResult {
    const lineStart = value.lastIndexOf('\n', start - 1) + 1;
    const before = value.slice(0, lineStart);
    const block = value.slice(lineStart, end);
    const after = value.slice(end);
    const lines = block.split('\n');
    const newBlock = lines.map(l => marker + l).join('\n');
    return {
        value: before + newBlock + after,
        start: start + marker.length,
        end: end + marker.length * lines.length,
    };
}

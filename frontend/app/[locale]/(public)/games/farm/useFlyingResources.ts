"use client";

import { useEffect } from 'react';
import type { FarmEvent } from './useFarmRoom';
import { ACTION_GAIN, ACTION_ICON, RES_ICON } from './farm-ui';

// 「拿資源」動畫：新 state 進來時，把取得的東西從行動格沿弧線飛到該玩家的資源格／農場格，落地彈一下並浮出 +N。
// 純 DOM（Web Animations API），不經 React state：起終點靠 data 屬性找
//   data-farm-action="<action>"、data-farm-res="<seat>:<res>"、data-farm-grid="<seat>"、data-farm-workers="<seat>"
// 同一目標可能有兩個元素（自己的資源在面板與手機橫幅各一份），挑目前在視窗內看得到的那個。

const MAX_FLYERS = 6;
const FLIGHT_MS = 650;
const STAGGER_MS = 70;

function visibleRect(el: Element): DOMRect | null {
    const r = el.getBoundingClientRect();
    if (r.width === 0 && r.height === 0) return null; // display:none（如 lg:hidden）
    const inView = r.bottom > 0 && r.top < window.innerHeight && r.right > 0 && r.left < window.innerWidth;
    return inView ? r : null;
}

/** 多個候選裡挑看得到的；都看不到就回第一個（仍可彈數字，只是不飛） */
function pickTarget(selector: string): { el: HTMLElement; rect: DOMRect | null } | null {
    const els = Array.from(document.querySelectorAll<HTMLElement>(selector));
    for (const el of els) {
        const rect = visibleRect(el);
        if (rect) return { el, rect };
    }
    return els[0] ? { el: els[0], rect: null } : null;
}

/** 二次貝茲曲線取樣成 keyframes（WAAPI 在 keyframe 間是直線內插，取多點才像弧線） */
function arcKeyframes(dx: number, dy: number): Keyframe[] {
    const cx = dx / 2;
    const cy = Math.min(0, dy) / 2 - 80; // 控制點往上拱
    return Array.from({ length: 9 }, (_, i) => {
        const t = i / 8;
        const x = 2 * (1 - t) * t * cx + t * t * dx;
        const y = 2 * (1 - t) * t * cy + t * t * dy;
        const scale = 1 + 0.25 * Math.sin(Math.PI * t) - 0.4 * t;
        return { transform: `translate(${x}px, ${y}px) scale(${scale})`, opacity: t > 0.9 ? 0.6 : 1 };
    });
}

function pop(el: HTMLElement, label: string | null) {
    el.animate(
        [{ transform: 'scale(1)' }, { transform: 'scale(1.4)' }, { transform: 'scale(1)' }],
        { duration: 350, easing: 'ease-out' },
    );
    if (!label) return;
    const r = el.getBoundingClientRect();
    const tag = document.createElement('span');
    tag.textContent = label;
    tag.className = 'pointer-events-none fixed z-50 text-sm font-bold text-primary-600 dark:text-primary-300';
    tag.style.left = `${r.left + r.width / 2}px`;
    tag.style.top = `${r.top}px`;
    document.body.appendChild(tag);
    tag.animate(
        [{ transform: 'translate(-50%, 0)', opacity: 1 }, { transform: 'translate(-50%, -28px)', opacity: 0 }],
        { duration: 900, easing: 'ease-out' },
    ).onfinish = () => tag.remove();
}

function fly(ev: Extract<FarmEvent, { kind: 'action' }>) {
    const gain = ACTION_GAIN[ev.action];
    if (!gain) return;
    const n = gain.kind === 'res' ? (gain.n ?? ev.amount) : 1;
    if (n <= 0) return;

    const target = pickTarget(
        gain.kind === 'res' ? `[data-farm-res="${ev.seat}:${gain.res}"]`
            : gain.kind === 'grid' ? `[data-farm-grid="${ev.seat}"]` : `[data-farm-workers="${ev.seat}"]`,
    );
    if (!target) return;
    const label = gain.kind === 'res' ? `+${n}` : null;

    const source = document.querySelector(`[data-farm-action="${ev.action}"]`);
    const from = source?.getBoundingClientRect();
    const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    // 減少動態 / 起終點有一端不在畫面上：不飛，只彈數字
    if (reduced || !from || !target.rect || (from.width === 0 && from.height === 0)) {
        pop(target.el, label);
        return;
    }

    const icon = gain.kind === 'res' ? RES_ICON[gain.res] : ACTION_ICON[ev.action];
    const sx = from.left + from.width / 2;
    const sy = from.top + from.height / 2;
    const tx = target.rect.left + target.rect.width / 2;
    const ty = target.rect.top + target.rect.height / 2;
    const count = Math.min(n, MAX_FLYERS);

    for (let i = 0; i < count; i++) {
        const jx = (Math.random() - 0.5) * 16; // 起點錯開一點，不要疊成一顆
        const jy = (Math.random() - 0.5) * 16;
        const el = document.createElement('span');
        el.textContent = icon;
        el.setAttribute('aria-hidden', 'true');
        el.className = 'pointer-events-none fixed z-50 text-2xl leading-none';
        el.style.left = `${sx + jx}px`;
        el.style.top = `${sy + jy}px`;
        el.style.marginLeft = '-0.5em';
        el.style.marginTop = '-0.5em';
        document.body.appendChild(el);
        const anim = el.animate(arcKeyframes(tx - sx - jx, ty - sy - jy), {
            duration: FLIGHT_MS, delay: i * STAGGER_MS, easing: 'ease-in-out', fill: 'backwards',
        });
        anim.onfinish = () => {
            el.remove();
            if (i === count - 1) pop(target.el, label);
        };
        anim.oncancel = () => el.remove();
    }
}

/// 每則新 state 播一次最新一批事件裡的放置動畫（自己與別人的都播）。
export function useFlyingResources(log: FarmEvent[], stateSeq: number) {
    useEffect(() => {
        for (const e of log) {
            if (e.seq === stateSeq && e.kind === 'action') fly(e);
        }
        // log 與 stateSeq 在同一個 WS handler 內更新；只在序號換了時播
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [stateSeq]);
}

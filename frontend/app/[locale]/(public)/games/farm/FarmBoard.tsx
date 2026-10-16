"use client";

import { useTranslations } from 'next-intl';
import { cn } from '@/libs/cn';
import { FARM_TILES, type PlayerState } from './farm-types';
import { ANIMAL_KEYS, HOUSE_ICON, RES_ICON, RES_KEYS, seatColor } from './farm-ui';

type Tile =
    | { kind: 'room' }
    | { kind: 'field'; crop: 'grain' | 'veg' | null; count: number }
    | { kind: 'pasture'; group: number; stable: boolean }
    | { kind: 'stable' }
    | { kind: 'free' };

/// 後端沒有逐格座標（phase-1），這裡依「房 → 田 → 牧場 → 散畜舍 → 空地」順序填滿 15 格，只是抽象示意。
function layoutTiles(p: PlayerState): Tile[] {
    const tiles: Tile[] = [];
    for (let i = 0; i < p.rooms; i++) tiles.push({ kind: 'room' });
    for (const f of p.fields) tiles.push({ kind: 'field', crop: f?.crop ?? null, count: f?.count ?? 0 });
    p.pastures.forEach((pa, group) => {
        for (let i = 0; i < pa.tiles; i++) tiles.push({ kind: 'pasture', group, stable: pa.stable && i === 0 });
    });
    for (let i = 0; i < p.loose_stables; i++) tiles.push({ kind: 'stable' });
    while (tiles.length < FARM_TILES) tiles.push({ kind: 'free' });
    return tiles.slice(0, FARM_TILES);
}

// 農地格底色屬遊戲資產色（同作物色，見 ARCHITECTURE「語意色保留」）
function tileBg(tile: Tile): string {
    switch (tile.kind) {
        case 'room': return 'bg-amber-100 dark:bg-amber-900/60';
        case 'field': return tile.crop ? 'bg-yellow-100 dark:bg-yellow-900/50' : 'bg-amber-200/70 dark:bg-amber-800/60'; // 已犁未播＝土色
        case 'pasture': return tile.group % 2 ? 'bg-green-100 dark:bg-green-900/50' : 'bg-lime-100 dark:bg-lime-900/50'; // 相鄰牧場交錯色
        case 'stable': return 'bg-orange-100 dark:bg-orange-900/50';
        case 'free': return 'border border-dashed border-neutral-300 dark:border-neutral-600';
    }
}

function FarmGrid({ p, size }: { p: PlayerState; size: 'lg' | 'sm' }) {
    const t = useTranslations('Farm');
    const tiles = layoutTiles(p);
    const label = (tile: Tile) => tile.kind === 'room' ? t(`house_${p.house}`)
        : tile.kind === 'field' ? (tile.crop ? `${t(`crop_${tile.crop}`)} ×${tile.count}` : t('tile_plowed'))
            : tile.kind === 'pasture' ? t('tile_pasture') + (tile.stable ? ` + ${t('stableTag')}` : '')
                : tile.kind === 'stable' ? t('stableTag') : t('tile_free');
    return (
        <ul className={cn('grid grid-cols-5', size === 'lg' ? 'gap-1' : 'gap-0.5')} aria-label={t('farmGridLabel')}>
            {tiles.map((tile, i) => (
                <li key={i} title={label(tile)} aria-label={label(tile)}
                    className={cn('relative flex aspect-square items-center justify-center rounded-md',
                        tileBg(tile), size === 'lg' ? 'text-2xl' : 'text-sm')}>
                    <span aria-hidden="true">
                        {tile.kind === 'room' ? HOUSE_ICON[p.house]
                            : tile.kind === 'field' ? (tile.crop === 'grain' ? '🌾' : tile.crop === 'veg' ? '🥕' : '')
                                : tile.kind === 'pasture' ? (tile.stable ? '🏚️' : '🌿')
                                    : tile.kind === 'stable' ? '🏚️' : ''}
                    </span>
                    {tile.kind === 'field' && tile.crop && (
                        <span className={cn('absolute bottom-0 right-0.5 font-mono font-bold text-neutral-700 dark:text-neutral-200', size === 'lg' ? 'text-xs' : 'text-[9px]')}>
                            {tile.count}
                        </span>
                    )}
                </li>
            ))}
        </ul>
    );
}

/// 單一玩家的農場面板：自己（lg）放大、其他人（sm）精簡。
export function PlayerPanel({
    p, seat, name, isMe, isCurrent, workersLeft, score, size,
}: {
    p: PlayerState; seat: number; name: string; isMe: boolean; isCurrent: boolean;
    workersLeft: number; score: number | undefined; size: 'lg' | 'sm';
}) {
    const t = useTranslations('Farm');
    const color = seatColor(seat);
    return (
        <section className={cn('flex flex-col gap-2 rounded-lg border-2 p-3 text-sm',
            isCurrent ? color.border : 'border-neutral-200 dark:border-neutral-700')}>
            <header className="flex flex-wrap items-center justify-between gap-2">
                <span className="flex min-w-0 items-center gap-1.5 font-semibold text-neutral-800 dark:text-neutral-100">
                    <span className={cn('h-3 w-3 flex-none rounded-full', color.dot)} />
                    <span className="truncate">{name}</span>
                    {isMe && <span className="text-xs text-primary-600 dark:text-primary-400">（{t('youTag')}）</span>}
                    {isCurrent && <span className={cn('rounded-full px-1.5 text-[11px] font-medium', color.text)}>▶ {t('currentTag')}</span>}
                </span>
                <span className="flex items-center gap-2">
                    {/* 工人：實心＝本輪還沒放 */}
                    <span className="flex gap-0.5" title={t('workersLeft', { n: workersLeft })} aria-label={t('workersLeft', { n: workersLeft })}>
                        {Array.from({ length: p.family }, (_, i) => (
                            <span key={i} className={cn('text-base leading-none', i < workersLeft ? '' : 'opacity-25 grayscale')} aria-hidden="true">👤</span>
                        ))}
                    </span>
                    {score !== undefined && (
                        <span className="rounded-full bg-neutral-100 px-2 py-0.5 text-xs font-semibold text-neutral-700 dark:bg-neutral-700 dark:text-neutral-100" title={t('scorePreviewHint')}>
                            {t('scorePreview', { n: score })}
                        </span>
                    )}
                </span>
            </header>

            <FarmGrid p={p} size={size} />

            <div className={cn('flex flex-wrap gap-x-2.5 gap-y-1', size === 'lg' ? 'text-sm' : 'text-xs')}>
                {[...RES_KEYS, ...ANIMAL_KEYS].map(k => (
                    <span key={k} className={cn('flex items-center gap-0.5', p[k] === 0 && 'opacity-35')}
                        title={k === 'sheep' || k === 'boar' || k === 'cattle' ? t(`animal_${k}`) : t(`res_${k}`)}>
                        <span aria-hidden="true">{RES_ICON[k]}</span>
                        <span className="font-mono font-medium">{p[k]}</span>
                    </span>
                ))}
            </div>

            {p.begging > 0 && <p className="text-xs font-semibold text-red-600 dark:text-red-400">{t('begging', { n: p.begging })}</p>}
        </section>
    );
}

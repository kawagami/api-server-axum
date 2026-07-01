import Image from 'next/image';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeSlug from 'rehype-slug';
import rehypeHighlight from 'rehype-highlight';
import { getTranslations } from 'next-intl/server';
import { extractHeadings } from '@/libs/blog-markdown';
import 'highlight.js/styles/github-dark.css';   // 深色高亮主題，配 prose 的深色 pre 底（亮/暗模式皆一致）

export default async function BlogArticle({ markdown }: { markdown: string }) {
    const headings = extractHeadings(markdown);
    const t = await getTranslations('BlogArticle');

    return (
        <div className="w-full h-[calc(100svh-120px)] overflow-auto scroll-smooth px-4 py-6">
            <div className="max-w-5xl mx-auto flex gap-8">
                <article className="prose prose-stone dark:prose-invert max-w-prose min-w-0 flex-1 [&_:is(h1,h2,h3,h4,h5,h6)]:scroll-mt-4">
                    <ReactMarkdown
                        remarkPlugins={[remarkGfm]}
                        rehypePlugins={[rehypeSlug, rehypeHighlight]}
                        components={{
                            img: ({ src, alt }) => (
                                <Image
                                    src={typeof src === 'string' ? src : ''}
                                    alt={alt || ''}
                                    width={800}
                                    height={600}
                                    style={{ width: 'auto', height: 'auto', maxWidth: '100%' }}
                                />
                            )
                        }}
                    >
                        {markdown}
                    </ReactMarkdown>
                </article>

                {headings.length > 0 && (
                    <aside className="hidden lg:block w-56 shrink-0">
                        <nav className="sticky top-0 text-sm">
                            <p className="font-semibold text-neutral-500 dark:text-neutral-400 mb-2">{t('toc')}</p>
                            <ul className="space-y-1.5 border-l border-neutral-200 dark:border-neutral-700">
                                {headings.map((h, i) => (
                                    <li key={i} style={{ paddingLeft: `${(h.level - 1) * 0.75 + 0.75}rem` }}>
                                        <a
                                            href={`#${h.slug}`}
                                            className="block text-neutral-500 dark:text-neutral-400 hover:text-primary-600 dark:hover:text-primary-400 transition-colors"
                                        >
                                            {h.text}
                                        </a>
                                    </li>
                                ))}
                            </ul>
                        </nav>
                    </aside>
                )}
            </div>
        </div>
    );
}

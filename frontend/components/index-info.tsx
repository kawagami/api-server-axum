import { Link } from "@/i18n/navigation";

interface Props {
    href: string;
    title: string;
    contents: React.ReactNode[];
    introLabel: string;
}

export default function IndexInfo({ href, title, contents, introLabel }: Props) {
    return (
        <div className={`p-4 sm:p-8 border-4 rounded-3xl border-double border-neutral-800 dark:border-neutral-200 ${href === '/' ? 'sm:col-span-2' : ''} bg-white dark:bg-neutral-900 shadow-lg hover:shadow-2xl transition-shadow duration-300`}>
            <div className="text-2xl sm:text-4xl font-semibold text-primary-700 dark:text-primary-300 hover:text-primary-500 transition-colors">
                <Link href={href} className="hover:text-primary-500">{title}</Link>
                <span className="text-lg text-neutral-500 ml-3">{introLabel}</span>
            </div>
            <div className="mt-6 text-neutral-800 dark:text-neutral-300 space-y-4 leading-relaxed">
                {contents.map((content, index) => (
                    <div key={`content_index_${index}`} className="border-b border-neutral-300 dark:border-neutral-700 pb-3 last:border-none">
                        {content}
                    </div>
                ))}
            </div>
        </div>
    );
}

import { Link } from "@/i18n/navigation";
import { useTranslations } from "next-intl";
import ShowClientTime from "@/components/blogs/show-client-time";

interface Props {
    id: string;
    toc: string;
    tags: string[];
    created_at: string;
    updated_at: string;
}

export default function BlogListCard({ id, toc, tags, created_at, updated_at }: Props) {
    const t = useTranslations("BlogListCard");

    // updated 只在有值且與 created 不同時顯示
    const showUpdated = updated_at && updated_at !== created_at;

    return (
        <Link
            href={`/blogs/${id}`}
            className="block bg-white dark:bg-neutral-800 shadow-md rounded-xl hover:shadow-lg transition-shadow duration-300 m-4"
        >
            <div className="p-4">
                <h2 className="text-center text-xl font-semibold text-neutral-800 dark:text-neutral-100 mb-2">
                    {toc || `Blog Post #${id}`}
                </h2>
                <div className="flex flex-wrap justify-center gap-2 mb-3">
                    {tags.map((tag) => (
                        <span
                            key={tag}
                            className="bg-primary-100 dark:bg-primary-900 text-primary-600 dark:text-primary-300 text-xs font-semibold px-2.5 py-0.5 rounded"
                        >
                            {tag}
                        </span>
                    ))}
                </div>
                <div className="text-neutral-400 dark:text-neutral-500 text-xs text-right">
                    {created_at && (
                        <p>
                            <strong>{t("created")}:</strong>{" "}
                            <ShowClientTime datetimeString={created_at} />
                        </p>
                    )}
                    {showUpdated && (
                        <p>
                            <strong>{t("updated")}:</strong>{" "}
                            <ShowClientTime datetimeString={updated_at} />
                        </p>
                    )}
                </div>
            </div>
        </Link>
    );
}

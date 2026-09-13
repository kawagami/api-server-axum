import { getTranslations } from "next-intl/server";
import { ArrowRight } from "lucide-react";
import { Link } from "@/i18n/navigation";
import ShowClientTime from "@/components/blogs/show-client-time";
import { getBlogs } from "@/api/blogs";
import { makeExcerpt } from "@/libs/blog-excerpt";

const LIMIT = 3;

// 首頁「最新文章」：純展示，抓最新 3 篇。
// 後端掛掉不能讓首頁跟著 500，故 catch 後回空陣列；沒文章就整段不渲染。
export default async function LatestPosts() {
    const [t, blogs] = await Promise.all([
        getTranslations("Home"),
        getBlogs({ page: 1, per_page: LIMIT })
            .then((res) => res.data)
            .catch(() => []),
    ]);

    if (blogs.length === 0) return null;

    return (
        <section className="flex flex-col gap-4">
            <div className="flex items-baseline justify-between gap-3">
                <h2 className="text-xl font-semibold text-neutral-800 dark:text-neutral-100">
                    {t("latest.title")}
                </h2>
                <Link
                    href="/blogs"
                    className="group inline-flex items-center gap-1 text-sm text-primary-600 dark:text-primary-300 hover:underline underline-offset-4"
                >
                    {t("latest.viewAll")}
                    <ArrowRight size={14} className="transition-transform group-hover:translate-x-0.5 motion-reduce:transition-none" />
                </Link>
            </div>

            <ul className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                {blogs.map((blog) => {
                    const title = blog.tocs[0] || t("latest.untitled");
                    return (
                        <li key={blog.id} className="flex">
                            <Link
                                href={`/blogs/${blog.id}`}
                                className="group flex w-full flex-col gap-2 rounded-xl bg-white dark:bg-neutral-800 p-5 shadow-md hover:shadow-lg transition-shadow duration-300"
                            >
                                <h3 className="line-clamp-2 font-semibold text-neutral-800 dark:text-neutral-100 group-hover:text-primary-600 dark:group-hover:text-primary-300 transition-colors">
                                    {title}
                                </h3>
                                <p className="line-clamp-2 text-sm leading-relaxed text-neutral-600 dark:text-neutral-400">
                                    {makeExcerpt(blog.markdown ?? "", blog.tocs[0] ?? "")}
                                </p>
                                <div className="mt-auto flex flex-wrap items-center gap-2 pt-1">
                                    {blog.created_at && (
                                        <ShowClientTime
                                            datetimeString={blog.created_at}
                                            variant="date"
                                            className="text-xs text-neutral-400 dark:text-neutral-500"
                                        />
                                    )}
                                    {(blog.tags ?? []).slice(0, 2).map((tag) => (
                                        <span
                                            key={tag}
                                            className="rounded-sm bg-primary-100 dark:bg-primary-900 px-2 py-0.5 text-xs font-semibold text-primary-600 dark:text-primary-300"
                                        >
                                            {tag}
                                        </span>
                                    ))}
                                </div>
                            </Link>
                        </li>
                    );
                })}
            </ul>
        </section>
    );
}

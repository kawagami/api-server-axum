import Link from "next/link";
import GithubMark from "@/components/github-mark";

export default function Footer() {
    return (
        <footer className="min-h-[50px] flex justify-center gap-4 text-center">
            <Link className="hover:scale-90" target="_blank" href="https://github.com/kawagami">
                <GithubMark className="text-neutral-900 dark:text-white" />
            </Link>
        </footer>
    );
}

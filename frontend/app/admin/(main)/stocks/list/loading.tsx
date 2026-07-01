import { Loader2 } from "lucide-react";

export default function Loading() {
    return (
        <div className="w-full flex items-center justify-center p-6 bg-neutral-100 dark:bg-neutral-800">
            <Loader2 className="w-8 h-8 animate-spin text-primary-500" />
        </div>
    );
}

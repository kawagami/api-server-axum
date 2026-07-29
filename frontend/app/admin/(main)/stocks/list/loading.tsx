import { Loader2 } from "lucide-react";

// 外層 padding / 底色由 admin layout 給，這裡只放置中的 spinner
export default function Loading() {
    return (
        <div className="w-full flex items-center justify-center py-16">
            <Loader2 className="w-8 h-8 animate-spin text-primary-500" />
        </div>
    );
}

import { ListTableSkeleton } from "@/components/loading/table-skeleton";

export default function Loading() {
    return (
        <div className="w-full max-w-5xl mx-auto">
            <ListTableSkeleton headers={["名稱", "狀態", "大小", "進度", "建立時間", "操作"]} />
        </div>
    );
}

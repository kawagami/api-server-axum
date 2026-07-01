import { ListTableSkeleton } from "@/components/loading/table-skeleton";

export default function Loading() {
    return <ListTableSkeleton headers={['ID', 'Level', 'Message', 'Target', 'File', 'Time']} rows={10} />;
}

import { ListTableSkeleton } from "@/components/loading/table-skeleton";

export default function Loading() {
    return <ListTableSkeleton headers={['Time', 'User', 'Method', 'Path', 'Query', 'Status']} rows={10} />;
}

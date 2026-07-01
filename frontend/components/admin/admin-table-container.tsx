export default function AdminTableContainer({ children }: { children: React.ReactNode }) {
    return (
        <div className="w-full h-[calc(100svh-180px)] overflow-auto p-3 sm:p-6">
            <div className="max-w-4xl mx-auto bg-white dark:bg-neutral-900 shadow-lg rounded-lg overflow-hidden">
                {children}
            </div>
        </div>
    );
}

export default function AdminTableContainer({ children }: { children: React.ReactNode }) {
    return (
        <div className="w-full">
            <div className="max-w-4xl mx-auto bg-white dark:bg-neutral-900 shadow-lg rounded-lg overflow-hidden">
                <div className="overflow-x-auto">
                    {children}
                </div>
            </div>
        </div>
    );
}

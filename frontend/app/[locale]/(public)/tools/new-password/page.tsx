"use client";

import { useState } from "react";
import { getNewPassword } from "@/api/tools";

export default function NewPasswordPage() {
    const [count, setCount] = useState(5);
    const [length, setLength] = useState(12);
    const [newPasswords, setNewPasswords] = useState<string[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [notification, setNotification] = useState("");

    const fetchNewPasswords = async () => {
        setIsLoading(true);
        try {
            const passwords = await getNewPassword(count, length);
            setNewPasswords(passwords);
        } catch {
            setNewPasswords(["Failed to fetch passwords"]);
        } finally {
            setIsLoading(false);
        }
    };

    const handleCopy = (password: string) => {
        navigator.clipboard.writeText(password)
            .then(() => {
                setNotification("密碼已複製");
                setTimeout(() => setNotification(""), 2000);
            })
            .catch(() => {
                setNotification("複製失敗");
                setTimeout(() => setNotification(""), 2000);
            });
    };

    const handleCountChange = (value: string) => setCount(Math.min(Math.max(Number(value), 1), 50));
    const handleLengthChange = (value: string) => setLength(Math.min(Math.max(Number(value), 1), 300));

    return (
        <div className="w-full h-[calc(100svh-120px)] text-center overflow-auto p-4">
            {notification && (
                <div className="fixed top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 bg-green-500 text-white px-6 py-3 rounded shadow-lg">
                    {notification}
                </div>
            )}
            <div className="flex flex-col items-center gap-2 mb-4">
                <div>
                    <label htmlFor="count" className="mr-2 dark:text-neutral-200">個數:</label>
                    <input id="count" type="number" value={count} onChange={(e) => handleCountChange(e.target.value)} className="border rounded px-2 py-1 w-20 dark:bg-neutral-700 dark:border-neutral-600 dark:text-neutral-100" />
                </div>
                <div>
                    <label htmlFor="length" className="mr-2 dark:text-neutral-200">長度:</label>
                    <input id="length" type="number" value={length} onChange={(e) => handleLengthChange(e.target.value)} className="border rounded px-2 py-1 w-20 dark:bg-neutral-700 dark:border-neutral-600 dark:text-neutral-100" />
                </div>
            </div>
            <button onClick={fetchNewPasswords} className="bg-primary-500 text-white px-4 py-2 rounded hover:bg-primary-600 dark:bg-primary-600 dark:hover:bg-primary-700" disabled={isLoading}>
                {isLoading ? "Loading..." : "Generate Passwords"}
            </button>
            <div className="mt-6">
                <h2 className="text-lg font-semibold dark:text-neutral-100">生成密碼:</h2>
                {isLoading ? (
                    <div className="mt-4 dark:text-neutral-300">Loading...</div>
                ) : newPasswords.length > 0 ? (
                    <ul className="mt-2 text-center">
                        {newPasswords.map((password, index) => (
                            <li key={index} className="mb-1 flex justify-center items-center gap-2">
                                <span className="dark:text-neutral-200">{password}</span>
                                <button onClick={() => handleCopy(password)} className="bg-neutral-200 text-neutral-800 px-2 py-1 rounded hover:bg-neutral-300 dark:bg-neutral-600 dark:text-neutral-100 dark:hover:bg-neutral-500">複製</button>
                            </li>
                        ))}
                    </ul>
                ) : (
                    <p className="dark:text-neutral-400">尚未生成密碼</p>
                )}
            </div>
        </div>
    );
}

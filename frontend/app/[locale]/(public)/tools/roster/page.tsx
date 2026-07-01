"use client";

import { useState } from "react";
import { CalendarDays, Send, Loader2, UserPlus, Trash2 } from "lucide-react";
import ShiftBadge from "./shift-badge";
import { postRoster } from "@/api/tools";

interface RosterEntry {
    id: string | number;
    name: string;
    shifts: string[];
}

export default function RosterPage() {
    const [names, setNames] = useState<string[]>(["人員1", "人員2"]);
    const [newName, setNewName] = useState("");
    const [days, setDays] = useState(31);
    const [rule, setRule] = useState("fairness");
    const [loading, setLoading] = useState(false);
    const [rosterData, setRosterData] = useState<RosterEntry[] | null>(null);

    const addName = () => {
        if (newName.trim()) {
            if (names.includes(newName.trim())) { alert("姓名重複了"); return; }
            setNames([...names, newName.trim()]);
            setNewName("");
        }
    };

    const removeName = (index: number) => setNames(names.filter((_, i) => i !== index));

    const handleGenerate = async () => {
        if (names.length === 0) { alert("請至少輸入一位人員"); return; }
        setLoading(true);
        try {
            const response = await postRoster({ names, days, rule });
            if (response?.data) {
                setRosterData(response.data as RosterEntry[]);
            } else {
                throw new Error("回傳資料格式不正確");
            }
        } catch {
            alert("排班失敗，請確認 Rust Server 是否正常運作");
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="w-full max-w-6xl px-4 pb-10 space-y-8">
            <section className="bg-white/60 dark:bg-neutral-800/60 backdrop-blur-md p-6 rounded-3xl shadow-lg border border-white/20">
                <h2 className="text-xl font-bold mb-4 flex items-center gap-2">
                    <CalendarDays className="text-primary-500" /> 排班參數設定
                </h2>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div className="space-y-3">
                        <label className="text-sm font-medium text-neutral-700 dark:text-neutral-300">參與人員 ({names.length})</label>
                        <div className="flex gap-2">
                            <input value={newName} onChange={(e) => setNewName(e.target.value)} onKeyDown={(e) => e.key === 'Enter' && addName()} placeholder="輸入姓名..." className="flex-1 px-4 py-2 rounded-xl border focus:ring-2 focus:ring-primary-500 outline-none dark:bg-neutral-700 dark:border-neutral-600" />
                            <button onClick={addName} className="p-2 bg-primary-500 text-white rounded-xl hover:bg-primary-600 transition-colors"><UserPlus size={20} /></button>
                        </div>
                        <div className="flex flex-wrap gap-2 mt-2 max-h-32 overflow-y-auto p-1">
                            {names.map((name, i) => (
                                <span key={i} className="px-3 py-1 bg-white/80 dark:bg-neutral-600 rounded-full text-sm flex items-center gap-2 shadow-sm border border-neutral-100 dark:border-neutral-500">
                                    {name}
                                    <button onClick={() => removeName(i)} className="hover:text-red-500 transition-colors"><Trash2 size={14} /></button>
                                </span>
                            ))}
                        </div>
                    </div>
                    <div className="space-y-4">
                        <div>
                            <label className="text-sm font-medium text-neutral-700 dark:text-neutral-300">排班天數 (1-31 天)</label>
                            <input type="number" min="1" max="31" value={days} onChange={(e) => setDays(Math.min(31, Math.max(1, parseInt(e.target.value) || 1)))} className="w-full mt-1 px-4 py-2 rounded-xl border dark:bg-neutral-700 dark:border-neutral-600 outline-none focus:ring-2 focus:ring-primary-500" />
                        </div>
                        <div>
                            <label className="text-sm font-medium text-neutral-700 dark:text-neutral-300">排班規則</label>
                            <select value={rule} onChange={(e) => setRule(e.target.value)} className="w-full mt-1 px-4 py-2 rounded-xl border dark:bg-neutral-700 dark:border-neutral-600 outline-none focus:ring-2 focus:ring-primary-500">
                                <option value="fairness">平均分配 (公平)</option>
                                <option value="morning_heavy">早班優先</option>
                                <option value="night_heavy">晚班優先</option>
                            </select>
                        </div>
                    </div>
                </div>
                <button onClick={handleGenerate} disabled={loading || names.length === 0} className="w-full mt-6 py-3 bg-gradient-to-r from-primary-600 to-primary-700 text-white rounded-2xl font-bold shadow-lg flex items-center justify-center gap-2 hover:opacity-90 disabled:opacity-50 transition-all active:scale-[0.98]">
                    {loading ? <Loader2 className="animate-spin" /> : <Send size={18} />}
                    {loading ? "Rust 計算中..." : "開始自動排班"}
                </button>
            </section>
            {rosterData && (
                <section className="bg-white/70 dark:bg-neutral-900/70 backdrop-blur-lg rounded-3xl shadow-xl border border-white/30 overflow-hidden">
                    <div className="overflow-x-auto">
                        <table className="w-full text-left border-collapse">
                            <thead>
                                <tr className="bg-neutral-100/50 dark:bg-neutral-800/50">
                                    <th className="p-4 border-b dark:border-neutral-700 font-semibold sticky left-0 bg-white/95 dark:bg-neutral-800/95 z-20">人員名單</th>
                                    {rosterData[0]?.shifts.map((_, i) => (
                                        <th key={i} className="p-4 border-b dark:border-neutral-700 font-semibold text-center min-w-[90px]">
                                            <div className="text-xs text-neutral-500 font-mono">DAY {i + 1}</div>
                                        </th>
                                    ))}
                                </tr>
                            </thead>
                            <tbody>
                                {rosterData.map((staff) => (
                                    <tr key={staff.id} className="hover:bg-white/40 dark:hover:bg-neutral-800/40 transition-colors">
                                        <td className="p-4 border-b dark:border-neutral-700 font-bold sticky left-0 bg-white/95 dark:bg-neutral-800/95 z-10">{staff.name}</td>
                                        {staff.shifts.map((shift, idx) => (
                                            <td key={idx} className="p-2 border-b dark:border-neutral-700 text-center"><ShiftBadge type={shift} /></td>
                                        ))}
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                </section>
            )}
        </div>
    );
}

-- hypervisor 抽走 vCPU 的時間佔比,與 cpu_pct(guest 真的在算東西)分開追蹤。
-- 混在一起算的話「機器很忙」與「機器被鄰居搶走」長得一模一樣,查不出來。
-- 既有 row 無此量測,補 0(前端視為缺值)。
ALTER TABLE public.system_metrics
    ADD COLUMN cpu_steal_pct real NOT NULL DEFAULT 0;

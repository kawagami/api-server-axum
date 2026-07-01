import { patchStockPendingAction } from "@/app/admin/(main)/stocks/actions";
import { Td } from "@/components/stocks/table-cells";
import type { StockChange } from "@/types";

export default function StockTableRow({ stock }: { stock: StockChange }) {
    return (
        <tr className="text-center dark:text-neutral-200">
            <Td>{stock.stock_no}</Td>
            <Td>{stock.stock_name}</Td>
            <Td>{stock.status}</Td>
            <Td>{stock.start_date}</Td>
            <Td>{stock.start_price}</Td>
            <Td>{stock.end_date}</Td>
            <Td>{stock.end_price}</Td>
            <Td className={stock.change < 0 ? "text-green-600 dark:text-green-400" : "text-red-600 dark:text-red-400"}>
                {stock.change ? `${stock.change}%` : ``}
            </Td>
            <Td>
                <form action={patchStockPendingAction} className="inline">
                    <input type="hidden" name="id" value={String(stock.id)} />
                    <button type="submit" className="bg-yellow-500 hover:bg-yellow-600 text-white px-3 py-1 rounded mr-2">
                        再查詢
                    </button>
                </form>
            </Td>
        </tr>
    );
}

import { flexRender, Row } from "@tanstack/react-table";

import { Task } from "@/features/tasks/types/Task";

interface TaskTableRowProps {
  row: Row<Task>;
}

export default function TaskTableRow({ row }: TaskTableRowProps) {
  return (
    <tr key={row.id} className="border-b border-secondary-text/20">
      {row.getVisibleCells().map((cell) => (
        <td key={cell.id} className="py-6 px-4">
          {flexRender(cell.column.columnDef.cell, cell.getContext())}
        </td>
      ))}
    </tr>
  );
}

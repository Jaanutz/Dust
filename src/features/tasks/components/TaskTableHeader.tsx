import { flexRender, HeaderGroup } from "@tanstack/react-table";

import { Task } from "@/features/tasks/types/Task";

interface TaskTableHeaderProps {
  headerGroup: HeaderGroup<Task>;
}

export default function TaskTableHeader({ headerGroup }: TaskTableHeaderProps) {
  return (
    <tr key={headerGroup.id}>
      {headerGroup.headers.map((header) => (
        <th
          key={header.id}
          className={`py-4 px-4 font-normal ${header.column.columnDef.meta?.className || ""}`}
        >
          {flexRender(header.column.columnDef.header, header.getContext())}
        </th>
      ))}
    </tr>
  );
}

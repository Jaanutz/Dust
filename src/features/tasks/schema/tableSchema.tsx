import { createColumnHelper } from "@tanstack/react-table";
import { ReactNode } from "react";

import TaskProgress from "@/features/tasks/components/TaskProgress";
import TaskSelectionCheckbox from "@/features/tasks/components/TaskSelectionCheckbox";
import TaskStateBadge from "@/features/tasks/components/TaskStateBadge";
import { type Task } from "@/features/tasks/types/Task";
import { formatBytes } from "@/utils/bytes";

declare module "@tanstack/react-table" {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  interface ColumnMeta<TData, TValue> {
    className?: string;
  }
}

const columnHelper = createColumnHelper<Task>();

type ColumnOptions =
  | {
      cell: (row: Task) => ReactNode;
      formatter?: never;
      className?: string;
    }
  | {
      formatter?: (row: Task) => string;
      cell?: never;
      className?: string;
    };

const createColumn = (
  id: keyof Task,
  header: string,
  options?: ColumnOptions,
) => {
  const { cell, formatter } = options || {};

  return columnHelper.accessor(id, {
    header: () => <h2 className="text-left">{header}</h2>,
    cell: (info) => {
      if (cell) {
        return cell(info.row.original);
      }

      return formatter ? formatter(info.row.original) : info.getValue();
    },
    meta: {
      className: options?.className || "",
    },
  });
};

export const COLUMNS = [
  createColumn("filename", "Filename", {
    className: "w-[30%]",
  }),
  createColumn("progress", "Progress", {
    cell: (row) => {
      return <TaskProgress progress={row.progress ?? 0} />;
    },
    className: "w-[25%]",
  }),
  createColumn("total_bytes", "Size", {
    formatter: (row) => formatBytes(row.total_bytes ?? 0),
    className: "w-[15%]",
  }),
  createColumn("state", "State", {
    cell: (row) => <TaskStateBadge state={row.state} />,
  }),
  columnHelper.display({
    id: "select",
    header: ({ table }) => (
      <TaskSelectionCheckbox
        isChecked={table.getIsAllRowsSelected()}
        onChange={(checked) => table.toggleAllRowsSelected(checked)}
      />
    ),
    cell: ({ row }) => (
      <TaskSelectionCheckbox
        isChecked={row.getIsSelected()}
        onChange={(checked) => row.toggleSelected(checked)}
      />
    ),
    meta: {
      className: "w-15",
    },
  }),
];

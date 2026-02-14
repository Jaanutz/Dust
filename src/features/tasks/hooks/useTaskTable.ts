import { getCoreRowModel, Table, useReactTable } from "@tanstack/react-table";
import { useMemo, useState } from "react";

import { COLUMNS } from "@/features/tasks/schema/tableSchema";
import { Task } from "@/features/tasks/types/Task";
import { TaskState } from "@/features/tasks/types/TaskState";
import { useTaskStore } from "@/stores/taskStore";
import { useViewStore } from "@/stores/viewStore";
import { TaskView } from "@/types/TaskView";

import { useTaskPolling } from "./useTaskPolling";

export function useTaskTable(): Table<Task> {
  const tasks = useTaskStore((state) => state.tasks);
  const selectedView = useViewStore((state) => state.selectedView);
  useTaskPolling();

  const filteredTasks = useMemo(
    () =>
      tasks.filter((task) => {
        switch (selectedView) {
          case TaskView.All:
            return true;
          case TaskView.Running:
            return task.state === TaskState.Running;
          case TaskView.Suspended:
            return task.state === TaskState.Paused;
          case TaskView.Complete:
            return task.state === TaskState.Completed;
          case TaskView.Incompleted:
            return task.state !== TaskState.Completed;
        }
      }),
    [tasks, selectedView],
  );

  const [rowSelection, setRowSelection] = useState({});
  const table = useReactTable<Task>({
    data: filteredTasks,
    columns: COLUMNS,
    state: {
      rowSelection,
    },
    getCoreRowModel: getCoreRowModel(),
    onRowSelectionChange: setRowSelection,
    getRowId: (row) => row.hash,
  });

  return table;
}

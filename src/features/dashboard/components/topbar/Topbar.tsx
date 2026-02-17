import { invoke } from "@tauri-apps/api/core";
import { Pause, Play, Plus, RotateCw, Trash2 } from "lucide-react";
import { useState } from "react";

import { TaskTable } from "@/features/tasks/types/TaskTable";
import { useTaskStore } from "@/stores/taskStore";

import CurrentViewLabel from "./CurrentViewLabel";
import IconButton from "./IconButton";
import TaskCreationModal from "../modal/TaskCreationModal";

interface TopbarProps {
  table: TaskTable;
}

export default function Topbar({ table }: TopbarProps) {
  const setShouldPoll = useTaskStore((state) => state.setShouldPoll);
  const refresh = useTaskStore((state) => state.refresh);

  const [isModalOpen, setIsModalOpen] = useState(false);

  const handleTaskAction = async (
    action: "spawn_tasks" | "pause_tasks" | "remove_tasks" | "restart_tasks",
    options?: { poll?: boolean; refreshAfter?: boolean },
  ) => {
    const hashes = table
      .getCoreRowModel()
      .flatRows.filter((row) => row.getIsSelected())
      .map((row) => row.original.hash);
    if (!hashes.length) return;

    await invoke(action, { hashes });

    if (options?.poll) setShouldPoll(true);
    if (options?.refreshAfter) refresh();

    table.resetRowSelection();
  };

  const buttons = [
    {
      icon: <Plus className="scale-120" />,
      onClick: () => setIsModalOpen(true),
    },
    {
      icon: <Trash2 />,
      onClick: () => handleTaskAction("remove_tasks", { refreshAfter: true }),
    },
    {
      icon: <RotateCw />,
      onClick: () => handleTaskAction("restart_tasks", { poll: true }),
    },
    {
      icon: <Play />,
      onClick: () => handleTaskAction("spawn_tasks", { poll: true }),
    },
    {
      icon: <Pause />,
      onClick: () => handleTaskAction("pause_tasks", { refreshAfter: true }),
    },
  ];

  return (
    <div className="px-5 py-4 flex items-center gap-20 justify-between">
      <CurrentViewLabel numberOfTasks={table.getCoreRowModel().rows.length} />

      <div className="flex gap-3 items-center">
        {buttons.map((button, index) => (
          <IconButton key={index} icon={button.icon} onClick={button.onClick} />
        ))}
      </div>

      {isModalOpen && <TaskCreationModal close={() => setIsModalOpen(false)} />}
    </div>
  );
}

import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";

import { Task } from "@/features/tasks/types/Task";

interface TaskStore {
  tasks: Task[];
  shouldPoll: boolean;

  setTasks: (tasks: Task[]) => void;
  setShouldPoll: (value: boolean) => void;
  refresh: () => Promise<Task[]>;
}

export const useTaskStore = create<TaskStore>((set) => ({
  tasks: [],
  shouldPoll: false,

  setTasks: (tasks) => set({ tasks }),
  setShouldPoll: (value) => set({ shouldPoll: value }),

  refresh: async () => {
    try {
      const tasks = await invoke<Task[]>("get_tasks");

      set({ tasks });
      return tasks;
    } catch{
      return [];
    }
  },
}));

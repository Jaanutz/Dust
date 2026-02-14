import { create } from "zustand";

import { TaskView } from "@/types/TaskView";

interface ViewStore {
  selectedView: TaskView;
  setView: (view: TaskView) => void;
}

export const useViewStore = create<ViewStore>((set) => ({
  selectedView: TaskView.All,
  setView: (view: TaskView) => set({ selectedView: view }),
}));

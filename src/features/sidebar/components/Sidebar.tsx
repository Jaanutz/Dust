import { ChartPie, CheckCircle, Menu, Pause, Play } from "lucide-react";

import { useViewStore } from "@/stores/viewStore";
import { TaskView } from "@/types/TaskView";

import SidebarItem from "./SidebarItem";

const iconClassName = "w-5 h-5";
const SIDEBAR_OPTIONS = [
  {
    name: TaskView.All,
    icon: <Menu className={iconClassName} />,
  },
  {
    name: TaskView.Running,
    icon: <Play className={iconClassName} />,
  },
  {
    name: TaskView.Suspended,
    icon: <Pause className={iconClassName} />,
  },
  {
    name: TaskView.Complete,
    icon: <CheckCircle className={iconClassName} />,
  },
  {
    name: TaskView.Incompleted,
    icon: <ChartPie className={iconClassName} />,
  },
];

export default function Sidebar() {
  const viewState = useViewStore((state) => state);

  return (
    <aside className="pr-8 pl-2 w-60">
      <h2 className="ml-5">Tasks</h2>
      <ul className="mt-2 flex flex-col gap-1.75">
        {SIDEBAR_OPTIONS.map((item) => (
          <SidebarItem
            key={item.name}
            item={item}
            isSelected={viewState.selectedView === item.name}
            onClick={() => viewState.setView(item.name)}
          />
        ))}
      </ul>
    </aside>
  );
}

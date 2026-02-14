import TaskTable from "@/features/tasks/components/TaskTable";
import { useTaskTable } from "@/features/tasks/hooks/useTaskTable";

import Topbar from "./topbar/Topbar";

export default function Dashboard() {
  const table = useTaskTable();

  return (
    <section className="py-2 grow bg-primary-background rounded-xl outline-1 outline-secondary-text/20">
      <Topbar table={table} />
      <TaskTable table={table} />
    </section>
  );
}

import { TaskState } from "@/features/tasks/types/TaskState";

const STATE_COLORS: Record<TaskState, string> = {
  [TaskState.Running]: "bg-accent/10",
  [TaskState.Paused]: "bg-yellow-500/10",
  [TaskState.Completed]: "bg-green-500/10",
};

interface TaskStateBadgeProps {
  state: TaskState;
}

export default function TaskStateBadge({ state }: TaskStateBadgeProps) {
  return (
    <span className={`${STATE_COLORS[state]} px-3 py-0.5 rounded-lg`}>
      {state}
    </span>
  );
}

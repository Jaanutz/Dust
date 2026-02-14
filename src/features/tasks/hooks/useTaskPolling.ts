import { useEffect, useRef } from "react";

import { TaskState } from "@/features/tasks/types/TaskState";
import { useTaskStore } from "@/stores/taskStore";

const POLL_INTERVAL = 100;

export function useTaskPolling() {
  const refresh = useTaskStore((state) => state.refresh);
  const shouldPoll = useTaskStore((state) => state.shouldPoll);
  const setShouldPoll = useTaskStore((state) => state.setShouldPoll);

  const intervalRef = useRef<number | null>(null);

  useEffect(() => {
    if (intervalRef.current !== null) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }

    if (!shouldPoll) return;

    intervalRef.current = window.setInterval(async () => {
      console.log("Polling for task updates...");

      const tasks = await refresh();

      const hasRunningTask = tasks.some((t) => t.state === TaskState.Running);

      if (!hasRunningTask) {
        if (intervalRef.current !== null) {
          clearInterval(intervalRef.current);
          intervalRef.current = null;
        }
        setShouldPoll(false);
      }
    }, POLL_INTERVAL);

    return () => {
      if (intervalRef.current !== null) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
    };
  }, [shouldPoll, refresh, setShouldPoll]);
}

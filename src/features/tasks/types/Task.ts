import { type TaskState } from "./TaskState";

export interface Task {
  state: TaskState;
  bytes_received: number;
  filename: string;
  url: string;
  hash: string;

  total_bytes?: number;
  progress?: number;
  speed?: number;
}

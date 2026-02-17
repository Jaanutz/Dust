import { invoke } from "@tauri-apps/api/core";
import { CircleX } from "lucide-react";
import { useState } from "react";

import { useTaskStore } from "@/stores/taskStore";

import DirectorySelector from "./DirectoryPicker";
import TaskCreationModalInput from "./TaskCreationModalInput";

interface TaskCreationModalProps {
  close: () => void;
}

export default function TaskCreationModal({ close }: TaskCreationModalProps) {
  const refresh = useTaskStore((state) => state.refresh);
  const [directory, setDirectory] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const inputItems = [
    { label: "URL", placeholder: "Enter the URL" },
    { label: "Filename", placeholder: "Enter the filename" },
  ];

  async function handleSubmit(event: React.FormEvent) {
    event.preventDefault();
    const formData = new FormData(event.target as HTMLFormElement);

    try {
      const filename = String(formData.get(inputItems[1].label)).trim();
      const url = String(formData.get(inputItems[0].label));

      if (!url) {
        setErrorMessage("URL is required.");
        return;
      }

      if (!filename) {
        setErrorMessage("Filename is required.");
        return;
      }

      if (!directory) {
        setErrorMessage("Directory is required.");
        return;
      }

      await invoke("add_task", {
        filename,
        filePath: directory,
        url,
      });

      refresh();
      close();
    } catch (error) {
      setErrorMessage(String(error));
    }
  }

  return (
    <div className="z-10 fixed inset-0 bg-black/40 flex items-center justify-center">
      <div className="bg-primary-background p-6 rounded-lg w-lg">
        <div className="flex justify-between items-start">
          <h2 className="text-xl font-bold mb-4">New Task</h2>
          <button className="cursor-pointer" onClick={close}>
            <CircleX className="w-5.5 h-5.5  text-secondary-text/60" />
          </button>
        </div>
        <form className="flex flex-col" onSubmit={handleSubmit}>
          <div className="flex flex-col gap-6">
            {inputItems.map((item) => (
              <TaskCreationModalInput
                key={item.label}
                label={item.label}
                placeholder={item.placeholder}
              />
            ))}
            <DirectorySelector
              directory={directory}
              setDirectory={setDirectory}
            />
          </div>
          <div className="mt-6">
            {errorMessage && (
              <p className="text-sm text-red-500">{errorMessage}</p>
            )}
            <button
              type="submit"
              className="cursor-pointer mt-2 w-full bg-accent text-primary-background p-2 rounded-md"
            >
              Add Task
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

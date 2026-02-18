import { open } from "@tauri-apps/plugin-dialog";
import { Folder } from "lucide-react";

interface DirectorySelectorProps {
  directory: string | null;
  setDirectory: (directory: string | null) => void;
}

export default function DirectorySelector({
  directory,
  setDirectory,
}: DirectorySelectorProps) {
  const openDirectory = async () => {
    const result = await open({
      directory: true,
      multiple: false,
    });
    if (result === null) return;

    setDirectory(result);
  };

  return (
    <div>
      <label
        htmlFor={"fileDirectory"}
        className="block text-sm font-medium text-secondary-text"
      >
        File Directory
      </label>
      <div className="mt-2 flex items-center gap-2">
        <button
          className="cursor-pointer flex items-center justify-center p-0.5 h-7 w-7 bg-secondary-background rounded-sm outline outline-secondary-text/20"
          type="button"
          onClick={openDirectory}
        >
          <Folder className="w-5 h-5 text-secondary-text" />
        </button>
        <input
          disabled
          type="text"
          id={"fileDirectory"}
          name={"fileDirectory"}
          value={directory ? truncateMiddle(directory) : ""}
          className="py-0.5 px-3 block w-full bg-secondary-background text-base outline outline-secondary-text/20 rounded-sm focus:outline-offset-0"
          placeholder={"Choose the file directory"}
        />
      </div>
    </div>
  );
}

function truncateMiddle(value: string) {
  const maxLength = 45;
  if (value.length <= maxLength) return value;

  const endLength = maxLength;
  return "…" + value.slice(value.length - endLength);
}

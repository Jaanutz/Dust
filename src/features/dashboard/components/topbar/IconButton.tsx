import { ReactNode } from "react";

interface IconButtonProps {
  icon: ReactNode;
  onClick: () => void;
}

export default function IconButton({ icon, onClick }: IconButtonProps) {
  return (
    <button
      className="cursor-pointer w-9 h-9 p-1.75 flex items-center justify-center outline outline-secondary-text/20 rounded-xs transition-all duration-150 ease-out hover:shadow-lg active:translate-y-1 active:shadow-lg"
      onClick={onClick}
      type="button"
    >
      {icon}
    </button>
  );
}

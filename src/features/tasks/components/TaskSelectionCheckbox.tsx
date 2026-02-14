import { Check } from "lucide-react";

interface TaskSelectionCheckboxProps {
  isChecked: boolean;
  onChange: (checked: boolean) => void;
}

export default function TaskSelectionCheckbox({
  isChecked,
  onChange,
}: TaskSelectionCheckboxProps) {
  return (
    <div className="flex items-center w-fit">
      <label className="cursor-pointer  relative">
        <input
          type="checkbox"
          className="appearance-none h-6 w-6 rounded transition-colors duration-200 border border-secondary-text/50 checked:bg-accent checked:border-accent"
          checked={isChecked}
          onChange={(e) => onChange?.(e.target.checked)}
        />
        <Check className="absolute inset-0 text-sm text-white p-0.5" />
      </label>
    </div>
  );
}

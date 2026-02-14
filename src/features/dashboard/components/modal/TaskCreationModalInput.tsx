interface TaskCreationModalInputProps {
  label: string;
  placeholder: string;
}

export default function TaskCreationModalInput({
  label,
  placeholder,
}: TaskCreationModalInputProps) {
  return (
    <div>
      <label
        htmlFor={label}
        className="block text-sm font-medium text-secondary-text"
      >
        {label}
      </label>
      <input
        type="text"
        id={label}
        name={label}
        className="mt-2 py-0.5 px-3 block w-full bg-secondary-background text-base outline outline-secondary-text/20 rounded-sm focus:outline-offset-0"
        placeholder={placeholder}
      />
    </div>
  );
}

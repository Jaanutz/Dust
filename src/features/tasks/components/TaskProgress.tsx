interface TaskProgress {
  progress: number;
}

export default function TaskProgress({ progress }: TaskProgress) {
  const percentage = Math.round(progress * 100);

  return (
    <div className="relative w-full h-5 rounded-full bg-accent/5 outline outline-secondary-text/20 outline-offset-2">
      <div
        className="absolute top-0 left-0 h-full rounded-full bg-linear-to-r from-accent/50 to-accent transition-all duration-300 ease-in-out"
        style={{ width: `${percentage}%` }}
      />
      <div className="absolute inset-0 flex items-center justify-center text-sm pointer-events-none">
        {percentage}
      </div>
    </div>
  );
}

const OPACITY_TRANSITION_PERCENTAGE = 14;

interface TaskProgress {
  progress: number;
}

export default function TaskProgress({ progress }: TaskProgress) {
  const easedProgress = fastSlowFast(progress);
  const percentage = Math.round(easedProgress * 100);

  return (
    <div className="relative w-[85%] h-5 rounded-full bg-accent/5 outline outline-secondary-text/20 outline-offset-2">
      <div
        className="absolute top-0 left-0 h-full rounded-full bg-linear-to-r from-accent/50 to-accent transition-all duration-200 ease-in-out"
        style={{ width: `${percentage}%` }}
      >
        <div
          className={`absolute right-2 flex items-center justify-center text-sm pointer-events-none transition-colors  ${percentage < OPACITY_TRANSITION_PERCENTAGE ? "text-transparent" : "text-primary-background"}`}
        >
          {percentage}
        </div>
      </div>
    </div>
  );
}

// between 0 and 1
const CURVE_STRENGTH = 0.5;

function fastSlowFast(t: number) {
  const amplitude = CURVE_STRENGTH / (2 - Math.PI);
  return (
    (1 - 2 * amplitude) * t +
    amplitude +
    amplitude * Math.sin(Math.PI * (t - 0.5))
  );
}

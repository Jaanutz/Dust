import { useViewStore } from "@/stores/viewStore";

interface CurrentViewLabelProps {
  numberOfTasks: number;
}

export default function CurrentViewLabel({
  numberOfTasks,
}: CurrentViewLabelProps) {
  const selectedView = useViewStore((state) => state.selectedView);

  return (
    <div className="flex items-center gap-3.25">
      <h1 className="text-black text-2xl font-bold">{selectedView}</h1>
      <h2 className="w-8 h-8 text-2xl font-bold flex items-center justify-center bg-secondary-background text-secondary-text rounded-lg">
        {numberOfTasks}
      </h2>
    </div>
  );
}

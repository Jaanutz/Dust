import { ReactNode } from "react";

interface SidebarItem {
  name: string;
  icon: ReactNode;
}

interface SidebarItemProps {
  item: SidebarItem;
  isSelected: boolean;
  onClick: () => void;
}

export default function SidebarItem({
  item,
  isSelected,
  onClick,
}: SidebarItemProps) {
  const { name, icon } = item;

  return (
    <li
      key={name}
      className={`cursor-pointer px-4 py-1.5 flex items-center gap-3 rounded-md ${
        isSelected
          ? "bg-primary-background text-black outline outline-secondary-text/20"
          : "hover:text-black/50 hover:bg-primary-background/80 hover:outline hover:outline-secondary-text/10"
      }`}
      onClick={onClick}
    >
      {icon}
      {name}
    </li>
  );
}

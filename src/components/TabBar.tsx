import { X } from "lucide-react";

export interface Tab {
  id: string;
  label: string;
  type: "table" | "empty";
  tableName?: string;
}

interface TabBarProps {
  tabs: Tab[];
  activeTabId: string | null;
  onTabClick: (tabId: string) => void;
  onTabClose: (tabId: string) => void;
}

export function TabBar({
  tabs,
  activeTabId,
  onTabClick,
  onTabClose,
}: TabBarProps) {
  if (tabs.length === 0) {
    return null;
  }

  return (
    <div className="flex items-center h-10 bg-card border-b border-border overflow-x-auto">
      {tabs.map((tab) => {
        const isActive = tab.id === activeTabId;
        return (
          <div
            key={tab.id}
            className={`
              group flex items-center gap-2 px-4 h-full border-r border-border cursor-pointer
              ${isActive ? "bg-background" : "bg-card hover:bg-accent/50"}
              transition-colors relative
            `}
            onClick={() => onTabClick(tab.id)}
          >
            <span
              className={`text-sm ${isActive ? "text-foreground" : "text-muted-foreground"}`}
            >
              {tab.label}
            </span>
            <button
              onClick={(e) => {
                e.stopPropagation();
                onTabClose(tab.id);
              }}
              className="opacity-0 group-hover:opacity-100 hover:bg-accent rounded p-0.5 transition-opacity"
            >
              <X className="w-3 h-3" />
            </button>
            {isActive && (
              <div className="absolute bottom-0 left-0 right-0 h-0.5 bg-primary" />
            )}
          </div>
        );
      })}
    </div>
  );
}

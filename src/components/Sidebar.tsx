import { Database, Table } from "lucide-react";
import { ThemeToggle } from "./ThemeToggle";

interface SidebarProps {
  onTableClick: (tableName: string) => void;
}

const tables = [
  { name: "users", icon: Table },
  { name: "organizations", icon: Table },
];

export function Sidebar({ onTableClick }: SidebarProps) {
  return (
    <div className="w-64 h-full bg-card border-r border-border flex flex-col">
      {/* Header */}
      <div className="p-4 border-b border-border">
        <div className="flex items-center justify-between text-foreground">
          <div className="flex items-center gap-2">
            <Database className="w-5 h-5" />
            <h1 className="font-semibold text-lg">Daedalus</h1>
          </div>
          <ThemeToggle />
        </div>
      </div>

      {/* Tables Section */}
      <div className="flex-1 overflow-y-auto">
        <div className="p-2">
          <div className="px-3 py-2 text-xs font-semibold text-muted-foreground uppercase tracking-wide">
            Tables
          </div>
          <div className="space-y-1">
            {tables.map((table) => {
              const Icon = table.icon;
              return (
                <button
                  key={table.name}
                  onClick={() => onTableClick(table.name)}
                  className="w-full flex items-center gap-2 px-3 py-2 text-sm rounded-md text-foreground hover:bg-accent hover:text-accent-foreground transition-colors"
                >
                  <Icon className="w-4 h-4" />
                  <span>{table.name}</span>
                </button>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
}

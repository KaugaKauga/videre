import { Database } from "@/components/icons";

export function EmptyState() {
  return (
    <div className="flex-1 flex items-center justify-center bg-background">
      <div className="text-center">
        <Database className="w-16 h-16 text-muted-foreground mx-auto mb-4" />
        <h3 className="text-lg font-semibold text-foreground mb-2">
          No table selected
        </h3>
        <p className="text-sm text-muted-foreground max-w-sm">
          Select a table from the sidebar to view its contents
        </p>
      </div>
    </div>
  );
}

import { FileText } from "lucide-react";

export function EmptyTab() {
  return (
    <div className="flex-1 flex items-center justify-center bg-background">
      <div className="text-center max-w-md">
        <FileText className="w-16 h-16 text-muted-foreground mx-auto mb-4" />
        <h3 className="text-lg font-semibold text-foreground mb-2">
          Empty Tab
        </h3>
        <p className="text-sm text-muted-foreground mb-4">
          This is an empty tab. Select a table from the sidebar to view its contents,
          or use this space for custom queries.
        </p>
        <div className="text-xs text-muted-foreground bg-muted/50 rounded-md p-3 border border-border">
          <p className="font-medium mb-1">Keyboard Shortcuts:</p>
          <div className="space-y-1 text-left">
            <div className="flex justify-between gap-4">
              <span>New tab</span>
              <kbd className="px-2 py-0.5 bg-background border border-border rounded text-xs">
                ⌘/Ctrl + T
              </kbd>
            </div>
            <div className="flex justify-between gap-4">
              <span>Close tab</span>
              <kbd className="px-2 py-0.5 bg-background border border-border rounded text-xs">
                ⌘/Ctrl + W
              </kbd>
            </div>
            <div className="flex justify-between gap-4">
              <span>Switch to tab 1-9</span>
              <kbd className="px-2 py-0.5 bg-background border border-border rounded text-xs">
                ⌘/Ctrl + [1-9]
              </kbd>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

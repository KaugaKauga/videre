import { useEffect } from "react";
import {
  Database,
  Table,
  Settings,
  Plug,
  Loader,
  ListTree,
} from "@/components/icons";
import {
  Sidebar as SidebarRoot,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "../../components/ui/sidebar";
import { useDbStore } from "@/store/dbStore";

interface SidebarProps {
  onTableClick: (tableName: string, schema: string) => void;
  onSettingsClick: () => void;
  onConnectionClick: () => void;
  onIndexesClick: () => void;
}

export function Sidebar({
  onTableClick,
  onSettingsClick,
  onConnectionClick,
  onIndexesClick,
}: SidebarProps) {
  const { tables, isConnected, isLoading, fetchDatabaseMetadata } =
    useDbStore();

  useEffect(() => {
    if (isConnected && tables.length === 0) {
      fetchDatabaseMetadata();
    }
  }, [isConnected, tables.length, fetchDatabaseMetadata]);

  return (
    <SidebarRoot>
      <SidebarHeader>
        <div className="flex items-center justify-between px-2">
          <div className="flex items-center gap-2">
            <Database className="w-5 h-5" />
            <h1 className="font-semibold text-lg">Videre</h1>
          </div>
        </div>
      </SidebarHeader>

      <SidebarContent className="flex flex-col">
        <SidebarGroup className="flex-1 min-h-0 flex flex-col">
          <SidebarGroupLabel>Tables</SidebarGroupLabel>
          <SidebarGroupContent className="flex-1 overflow-y-auto min-h-0">
            {isLoading ? (
              <div className="flex items-center justify-center py-4">
                <Loader className="w-4 h-4 animate-spin text-muted-foreground" />
              </div>
            ) : tables.length === 0 ? (
              <div className="px-3 py-2 text-xs text-muted-foreground">
                {isConnected ? "No tables found" : "Not connected"}
              </div>
            ) : (
              <SidebarMenu>
                {tables.map((table) => (
                  <SidebarMenuItem key={`${table.schema}.${table.name}`}>
                    <SidebarMenuButton
                      onClick={() => onTableClick(table.name, table.schema)}
                    >
                      <Table />
                      <span>{table.name}</span>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                ))}
              </SidebarMenu>
            )}
          </SidebarGroupContent>
        </SidebarGroup>

        {/* Database-level items pinned at bottom of content */}
        <div className="border-t border-border mt-auto">
          <SidebarMenu className="p-2">
            <SidebarMenuItem>
              <SidebarMenuButton onClick={onIndexesClick}>
                <ListTree />
                <span>Indexes</span>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </div>
      </SidebarContent>

      <SidebarFooter>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton onClick={onConnectionClick}>
              <Plug />
              <span>Connection</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <SidebarMenuButton onClick={onSettingsClick}>
              <Settings />
              <span>Settings</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarFooter>
    </SidebarRoot>
  );
}

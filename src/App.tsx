import { useState, useMemo } from "react";
import { Sidebar } from "./components/Sidebar";
import { TabBar, Tab } from "./components/TabBar";
import { TablePage } from "./features/table/TablePage";
import { EmptyState } from "./components/EmptyState";
import { EmptyTab } from "./components/EmptyTab";
import { SettingsPage } from "./features/settings/SettingsPage";
import { ConnectionPage } from "./features/connection/ConnectionPage";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
import { SidebarProvider, SidebarInset } from "./components/ui/sidebar";

function App() {
  const [tabs, setTabs] = useState<Tab[]>([]);
  const [activeTabId, setActiveTabId] = useState<string | null>(null);

  const handleTableClick = (tableName: string, schema: string) => {
    // Check if tab already exists
    const existingTab = tabs.find(
      (tab) => tab.type === "table" && tab.tableName === tableName,
    );

    if (existingTab) {
      // If tab exists, just focus it
      setActiveTabId(existingTab.id);
      return;
    }

    // Check if active tab is empty - if so, reuse it
    const activeTab = tabs.find((tab) => tab.id === activeTabId);
    if (activeTab && activeTab.type === "empty") {
      // Reuse the active empty tab by updating it in place
      const updatedTabs = tabs.map((tab) =>
        tab.id === activeTabId
          ? {
              id: activeTabId, // Keep the same ID
              label: tableName,
              type: "table" as const,
              tableName: tableName,
              schema: schema,
            }
          : tab,
      );
      setTabs(updatedTabs);
      // Active tab ID stays the same
    } else {
      // Create new tab
      const newTab: Tab = {
        id: `table-${tableName}-${Date.now()}`,
        label: tableName,
        type: "table",
        tableName: tableName,
        schema: schema,
      };
      setTabs([...tabs, newTab]);
      setActiveTabId(newTab.id);
    }
  };

  const handleSettingsClick = () => {
    // Check if settings tab already exists
    const existingTab = tabs.find((tab) => tab.type === "settings");

    if (existingTab) {
      // If tab exists, just focus it
      setActiveTabId(existingTab.id);
      return;
    }

    // Check if active tab is empty - if so, reuse it
    const activeTab = tabs.find((tab) => tab.id === activeTabId);
    if (activeTab && activeTab.type === "empty") {
      // Reuse the active empty tab by updating it in place
      const updatedTabs = tabs.map((tab) =>
        tab.id === activeTabId
          ? {
              id: activeTabId, // Keep the same ID
              label: "Settings",
              type: "settings" as const,
            }
          : tab,
      );
      setTabs(updatedTabs);
      // Active tab ID stays the same
    } else {
      // Create new settings tab
      const newTab: Tab = {
        id: `settings-${Date.now()}`,
        label: "Settings",
        type: "settings" as any,
      };
      setTabs([...tabs, newTab]);
      setActiveTabId(newTab.id);
    }
  };

  const handleConnectionClick = () => {
    // Check if connection tab already exists
    const existingTab = tabs.find((tab) => tab.type === "connection");

    if (existingTab) {
      // If tab exists, just focus it
      setActiveTabId(existingTab.id);
      return;
    }

    // Check if active tab is empty - if so, reuse it
    const activeTab = tabs.find((tab) => tab.id === activeTabId);
    if (activeTab && activeTab.type === "empty") {
      // Reuse the active empty tab by updating it in place
      const updatedTabs = tabs.map((tab) =>
        tab.id === activeTabId
          ? {
              id: activeTabId, // Keep the same ID
              label: "Connection",
              type: "connection" as const,
            }
          : tab,
      );
      setTabs(updatedTabs);
      // Active tab ID stays the same
    } else {
      // Create new connection tab
      const newTab: Tab = {
        id: `connection-${Date.now()}`,
        label: "Connection",
        type: "connection" as any,
      };
      setTabs([...tabs, newTab]);
      setActiveTabId(newTab.id);
    }
  };

  const handleTabClick = (tabId: string) => {
    setActiveTabId(tabId);
  };

  const handleTabClose = (tabId: string) => {
    const newTabs = tabs.filter((tab) => tab.id !== tabId);
    setTabs(newTabs);

    // If we closed the active tab, set a new active tab
    if (activeTabId === tabId) {
      if (newTabs.length > 0) {
        // Set the last tab as active
        setActiveTabId(newTabs[newTabs.length - 1].id);
      } else {
        setActiveTabId(null);
      }
    }
  };

  const handleNewEmptyTab = () => {
    // Find the lowest available number for Untitled tabs
    const untitledNumbers = tabs
      .filter((tab) => tab.type === "empty")
      .map((tab) => {
        const match = tab.label.match(/^Untitled (\d+)$/);
        return match ? parseInt(match[1], 10) : 0;
      })
      .filter((num) => num > 0);

    // Find the lowest available number
    let newNumber = 1;
    while (untitledNumbers.includes(newNumber)) {
      newNumber++;
    }

    const newTab: Tab = {
      id: `empty-${newNumber}-${Date.now()}`,
      label: `Untitled ${newNumber}`,
      type: "empty",
    };
    setTabs([...tabs, newTab]);
    setActiveTabId(newTab.id);
  };

  const handleCloseActiveTab = () => {
    if (activeTabId) {
      handleTabClose(activeTabId);
    }
  };

  const handleSwitchToTab = (index: number) => {
    if (index >= 0 && index < tabs.length) {
      setActiveTabId(tabs[index].id);
    }
  };

  // Define keyboard shortcuts
  const shortcuts = useMemo(
    () => [
      // Cmd/Ctrl + T - New empty tab
      {
        key: "t",
        ctrlOrCmd: true,
        handler: () => handleNewEmptyTab(),
      },
      // Cmd/Ctrl + W - Close active tab
      {
        key: "w",
        ctrlOrCmd: true,
        handler: () => handleCloseActiveTab(),
      },
      // Cmd/Ctrl + 1-9 - Switch to tab by index
      ...Array.from({ length: 9 }, (_, i) => ({
        key: String(i + 1),
        ctrlOrCmd: true,
        handler: () => handleSwitchToTab(i),
      })),
    ],
    [tabs, activeTabId],
  );

  useKeyboardShortcuts(shortcuts);

  const activeTab = tabs.find((tab) => tab.id === activeTabId);

  const renderContent = () => {
    if (!activeTab) {
      return <EmptyState />;
    }

    if (activeTab.type === "table" && activeTab.tableName) {
      return (
        <TablePage tableName={activeTab.tableName} schema={activeTab.schema} />
      );
    }

    if (activeTab.type === "empty") {
      return <EmptyTab />;
    }

    if (activeTab.type === "settings") {
      return <SettingsPage />;
    }

    if (activeTab.type === "connection") {
      return <ConnectionPage />;
    }

    return <EmptyState />;
  };

  return (
    <SidebarProvider className="h-full">
      <Sidebar
        onTableClick={handleTableClick}
        onSettingsClick={handleSettingsClick}
        onConnectionClick={handleConnectionClick}
      />
      <SidebarInset className="h-full overflow-hidden">
        <div className="flex flex-col h-full overflow-hidden">
          {/* Tab Bar */}
          <TabBar
            tabs={tabs}
            activeTabId={activeTabId}
            onTabClick={handleTabClick}
            onTabClose={handleTabClose}
          />

          {/* Content Area */}
          <div className="flex flex-col flex-1 overflow-hidden">
            {renderContent()}
          </div>
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}

export default App;

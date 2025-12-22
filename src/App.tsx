import { useState, useMemo } from "react";
import { Sidebar } from "./components/Sidebar";
import { TabBar, Tab } from "./components/TabBar";
import { TableView } from "./components/TableView";
import { EmptyState } from "./components/EmptyState";
import { EmptyTab } from "./components/EmptyTab";
import { SettingsView } from "./components/SettingsView";
import { ConnectionPage } from "./components/ConnectionPage";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";

function App() {
  const [tabs, setTabs] = useState<Tab[]>([]);
  const [activeTabId, setActiveTabId] = useState<string | null>(null);
  const [tabCounter, setTabCounter] = useState(0);

  const handleTableClick = (tableName: string) => {
    // Check if tab already exists
    const existingTab = tabs.find(
      (tab) => tab.type === "table" && tab.tableName === tableName,
    );

    if (existingTab) {
      // If tab exists, just focus it
      setActiveTabId(existingTab.id);
    } else {
      // Create new tab
      const newTab: Tab = {
        id: `table-${tableName}-${Date.now()}`,
        label: tableName,
        type: "table",
        tableName: tableName,
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
    const newCounter = tabCounter + 1;
    const newTab: Tab = {
      id: `empty-${newCounter}-${Date.now()}`,
      label: `Untitled ${newCounter}`,
      type: "empty",
    };
    setTabs([...tabs, newTab]);
    setActiveTabId(newTab.id);
    setTabCounter(newCounter);
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
    [tabs, activeTabId, tabCounter],
  );

  useKeyboardShortcuts(shortcuts);

  const activeTab = tabs.find((tab) => tab.id === activeTabId);

  const renderContent = () => {
    if (!activeTab) {
      return <EmptyState />;
    }

    if (activeTab.type === "table" && activeTab.tableName) {
      return <TableView tableName={activeTab.tableName} />;
    }

    if (activeTab.type === "empty") {
      return <EmptyTab />;
    }

    if (activeTab.type === "settings") {
      return <SettingsView />;
    }

    if (activeTab.type === "connection") {
      return <ConnectionPage />;
    }

    return <EmptyState />;
  };

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-background">
      {/* Sidebar */}
      <Sidebar
        onTableClick={handleTableClick}
        onSettingsClick={handleSettingsClick}
        onConnectionClick={handleConnectionClick}
      />

      {/* Main Content Area */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Tab Bar */}
        <TabBar
          tabs={tabs}
          activeTabId={activeTabId}
          onTabClick={handleTabClick}
          onTabClose={handleTabClose}
        />

        {/* Content Area */}
        {renderContent()}
      </div>
    </div>
  );
}

export default App;

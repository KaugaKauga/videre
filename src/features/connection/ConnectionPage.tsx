import { useState } from "react";
import {
  Database,
  Loader,
  CheckCircle,
  XCircle,
  Trash,
} from "@/components/icons";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { db } from "@/lib/tauri";
import { useDbStore } from "@/store/dbStore";
import { useConnectionStore, SavedConnection } from "@/store/connectionStore";

type Status =
  | { type: "idle" }
  | { type: "testing" }
  | { type: "connecting" }
  | { type: "success"; message: string }
  | { type: "error"; message: string };

const DEFAULT_FORM_DATA = {
  host: "localhost",
  port: "5432",
  database: "",
  username: "",
  password: "",
};

export function ConnectionPage() {
  const [formData, setFormData] = useState(DEFAULT_FORM_DATA);
  const [status, setStatus] = useState<Status>({ type: "idle" });

  const { setConnected, fetchDatabaseMetadata } = useDbStore();
  const { connections, isLoaded, saveConnection, removeConnection } =
    useConnectionStore();

  const isLoading = status.type === "testing" || status.type === "connecting";

  const handleTestConnection = async () => {
    setStatus({ type: "testing" });
    try {
      const result = await db.testConnection(formData);
      setStatus(
        result.success
          ? { type: "success", message: result.message }
          : { type: "error", message: result.message },
      );
    } catch (error) {
      setStatus({
        type: "error",
        message: `Failed to test connection: ${error}`,
      });
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setStatus({ type: "connecting" });

    try {
      const result = await db.connect(formData);

      if (result.success) {
        setStatus({ type: "success", message: result.message });
        await saveConnection({
          host: formData.host,
          port: formData.port,
          database: formData.database,
          username: formData.username,
        });
        setConnected(true);
        await fetchDatabaseMetadata();
      } else {
        setStatus({ type: "error", message: result.message });
      }
    } catch (error) {
      setStatus({ type: "error", message: `Failed to connect: ${error}` });
    }
  };

  const selectConnection = (connection: SavedConnection) => {
    setFormData({
      host: connection.host,
      port: connection.port,
      database: connection.database,
      username: connection.username,
      password: "",
    });
    setStatus({ type: "idle" });
  };

  const updateField =
    (field: keyof typeof formData) =>
    (e: React.ChangeEvent<HTMLInputElement>) =>
      setFormData((prev) => ({ ...prev, [field]: e.target.value }));

  return (
    <div className="flex-1 h-full flex items-center justify-center bg-background p-4 overflow-auto">
      <div className="relative">
        {/* Recent Connections */}
        {isLoaded && connections.length > 0 && (
          <div className="absolute right-full mr-4 top-0 flex flex-col gap-1">
            <h2 className="ml-3 text-muted-foreground">Recents</h2>
            {connections.map((connection) => (
              <div key={connection.id} className="group flex items-center">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => selectConnection(connection)}
                  className="justify-start text-muted-foreground font-normal"
                >
                  {connection.database}
                </Button>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    removeConnection(connection.id);
                  }}
                  className="opacity-0 group-hover:opacity-100 p-1 hover:bg-destructive/10 rounded transition-opacity"
                  title="Remove connection"
                >
                  <Trash className="w-3 h-3 text-destructive" />
                </button>
              </div>
            ))}
          </div>
        )}

        {/* Connection Form */}
        <Card className="w-full max-w-md">
          <CardHeader className="space-y-1">
            <div className="flex items-center gap-2">
              <Database className="w-6 h-6 text-primary" />
              <CardTitle className="text-2xl">Connect to PostgreSQL</CardTitle>
            </div>
            <CardDescription>
              Enter your database connection details to get started
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="host">Host</Label>
                <Input
                  id="host"
                  placeholder="localhost"
                  value={formData.host}
                  onChange={updateField("host")}
                  required
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="port">Port</Label>
                <Input
                  id="port"
                  placeholder="5432"
                  value={formData.port}
                  onChange={updateField("port")}
                  required
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="database">Database</Label>
                <Input
                  id="database"
                  placeholder="my_database"
                  value={formData.database}
                  onChange={updateField("database")}
                  required
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="username">Username</Label>
                <Input
                  id="username"
                  placeholder="postgres"
                  value={formData.username}
                  onChange={updateField("username")}
                  required
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="password">Password</Label>
                <Input
                  id="password"
                  type="password"
                  placeholder="Enter your password"
                  value={formData.password}
                  onChange={updateField("password")}
                  required
                />
              </div>

              {(status.type === "success" || status.type === "error") && (
                <div
                  className={`flex items-center gap-2 p-3 rounded-md text-sm ${
                    status.type === "success"
                      ? "bg-green-500/10 text-green-600 dark:text-green-400"
                      : "bg-red-500/10 text-red-600 dark:text-red-400"
                  }`}
                >
                  {status.type === "success" ? (
                    <CheckCircle className="w-4 h-4" />
                  ) : (
                    <XCircle className="w-4 h-4" />
                  )}
                  <span>{status.message}</span>
                </div>
              )}

              <div className="flex gap-2">
                <Button
                  type="button"
                  className="flex-1"
                  onClick={handleTestConnection}
                  disabled={isLoading}
                >
                  {status.type === "testing" && (
                    <Loader className="mr-2 h-4 w-4 animate-spin" />
                  )}
                  Test Connection
                </Button>
                <Button type="submit" className="flex-1" disabled={isLoading}>
                  {status.type === "connecting" && (
                    <Loader className="mr-2 h-4 w-4 animate-spin" />
                  )}
                  Connect
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

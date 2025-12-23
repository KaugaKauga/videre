import { useState } from "react";
import { Database, Loader2, CheckCircle, XCircle } from "lucide-react";
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

interface ConnectionFormData {
  host: string;
  port: string;
  database: string;
  username: string;
  password: string;
}

export function ConnectionPage() {
  const [formData, setFormData] = useState<ConnectionFormData>({
    host: "localhost",
    port: "5432",
    database: "",
    username: "",
    password: "",
  });

  const { setConnected, fetchTables } = useDbStore();
  const [isConnecting, setIsConnecting] = useState(false);
  const [isTesting, setIsTesting] = useState(false);
  const [connectionStatus, setConnectionStatus] = useState<{
    type: "success" | "error" | null;
    message: string;
  }>({ type: null, message: "" });

  const handleTestConnection = async () => {
    setIsTesting(true);
    setConnectionStatus({ type: null, message: "" });

    try {
      const result = await db.testConnection(formData);
      setConnectionStatus({
        type: result.success ? "success" : "error",
        message: result.message,
      });
    } catch (error) {
      setConnectionStatus({
        type: "error",
        message: `Failed to test connection: ${error}`,
      });
    } finally {
      setIsTesting(false);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsConnecting(true);
    setConnectionStatus({ type: null, message: "" });

    try {
      const result = await db.connect(formData);
      setConnectionStatus({
        type: result.success ? "success" : "error",
        message: result.message,
      });

      if (result.success) {
        // Update global state
        setConnected(true);
        // Fetch tables
        await fetchTables();
      }
    } catch (error) {
      setConnectionStatus({
        type: "error",
        message: `Failed to connect: ${error}`,
      });
    } finally {
      setIsConnecting(false);
    }
  };

  const handleChange = (field: keyof ConnectionFormData, value: string) => {
    setFormData((prev) => ({ ...prev, [field]: value }));
  };

  return (
    <div className="flex items-center justify-center min-h-screen bg-background p-4">
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
                type="text"
                placeholder="localhost"
                value={formData.host}
                onChange={(e) => handleChange("host", e.target.value)}
                required
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="port">Port</Label>
              <Input
                id="port"
                type="text"
                placeholder="5432"
                value={formData.port}
                onChange={(e) => handleChange("port", e.target.value)}
                required
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="database">Database</Label>
              <Input
                id="database"
                type="text"
                placeholder="my_database"
                value={formData.database}
                onChange={(e) => handleChange("database", e.target.value)}
                required
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="username">Username</Label>
              <Input
                id="username"
                type="text"
                placeholder="postgres"
                value={formData.username}
                onChange={(e) => handleChange("username", e.target.value)}
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
                onChange={(e) => handleChange("password", e.target.value)}
                required
              />
            </div>

            {connectionStatus.type && (
              <div
                className={`flex items-center gap-2 p-3 rounded-md text-sm ${
                  connectionStatus.type === "success"
                    ? "bg-green-500/10 text-green-600 dark:text-green-400"
                    : "bg-red-500/10 text-red-600 dark:text-red-400"
                }`}
              >
                {connectionStatus.type === "success" ? (
                  <CheckCircle className="w-4 h-4" />
                ) : (
                  <XCircle className="w-4 h-4" />
                )}
                <span>{connectionStatus.message}</span>
              </div>
            )}

            <div className="flex gap-2">
              <Button
                type="button"
                variant="outline"
                className="flex-1"
                onClick={handleTestConnection}
                disabled={isTesting || isConnecting}
              >
                {isTesting && <Loader2 className="w-4 h-4 mr-2 animate-spin" />}
                Test Connection
              </Button>
              <Button
                type="submit"
                className="flex-1"
                disabled={isConnecting || isTesting}
              >
                {isConnecting && (
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                )}
                Connect
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}

import { useState } from "react";
import { Database } from "lucide-react";
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

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    // TODO: Hook up to backend
    console.log("Connection data:", formData);
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

            <Button type="submit" className="w-full">
              Connect
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}

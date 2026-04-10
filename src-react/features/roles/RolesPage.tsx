import { useState, useMemo } from "react";
import { useDbStore } from "@/store/dbStore";
import { Users, Loader } from "@/components/icons";
import { TablePrivilege } from "@/lib/tauri";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetDescription,
  SheetBody,
  SheetClose,
} from "@/components/ui/sheet";

function formatConnectionLimit(limit: number): string {
  return limit === -1 ? "unlimited" : String(limit);
}

function formatValidUntil(validUntil: string | null): string {
  if (!validUntil) return "—";
  try {
    const date = new Date(validUntil);
    return date.toLocaleDateString();
  } catch {
    return validUntil;
  }
}

// Compute permission summary for a role based on their table privileges
function getPermissionSummary(privileges: TablePrivilege[]): {
  summary: string;
  type: "superuser" | "read-write" | "read-only" | "mixed" | "none";
} {
  if (privileges.length === 0) {
    return { summary: "no table access", type: "none" };
  }

  let hasSelect = false;
  let hasWrite = false;

  for (const priv of privileges) {
    if (priv.privileges.includes("SELECT")) {
      hasSelect = true;
    }
    if (
      priv.privileges.includes("INSERT") ||
      priv.privileges.includes("UPDATE") ||
      priv.privileges.includes("DELETE")
    ) {
      hasWrite = true;
    }
  }

  const tableCount = privileges.length;
  const tableText = `${tableCount} table${tableCount !== 1 ? "s" : ""}`;

  if (hasWrite && hasSelect) {
    return { summary: `read-write (${tableText})`, type: "read-write" };
  } else if (hasSelect && !hasWrite) {
    return { summary: `read-only (${tableText})`, type: "read-only" };
  } else if (hasWrite && !hasSelect) {
    return { summary: `write-only (${tableText})`, type: "mixed" };
  }

  return { summary: `mixed (${tableText})`, type: "mixed" };
}

// Component to display role details in the sheet
function RoleDetails({
  roleName,
  isLoading,
}: {
  roleName: string | null;
  isLoading: boolean;
}) {
  const { roles, getPrivilegesForRole } = useDbStore();

  if (isLoading || !roleName) {
    return (
      <div className="flex items-center justify-center py-8">
        <Loader className="w-5 h-5 animate-spin mr-2" />
        <span className="text-muted-foreground">Loading...</span>
      </div>
    );
  }

  const role = roles.find((r) => r.role_name === roleName);
  const privileges = getPrivilegesForRole(roleName);

  if (!role) {
    return <p className="text-muted-foreground text-sm">Role not found</p>;
  }

  // Group privileges by schema.table for display
  const groupedPrivileges = useMemo(() => {
    const grouped: Record<string, string[]> = {};
    for (const priv of privileges) {
      const key = `${priv.table_schema}.${priv.table_name}`;
      grouped[key] = priv.privileges;
    }
    return grouped;
  }, [privileges]);

  return (
    <div className="space-y-6">
      {/* Role Properties */}
      <div>
        <h4 className="text-sm font-medium text-foreground mb-3">
          Role Properties
        </h4>
        <div className="space-y-2 text-sm">
          <div className="flex justify-between">
            <span className="text-muted-foreground">Can Login</span>
            <span className="text-foreground">
              {role.can_login ? "Yes" : "No"}
            </span>
          </div>
          <div className="flex justify-between">
            <span className="text-muted-foreground">Superuser</span>
            <span
              className={
                role.is_superuser
                  ? "text-red-600 dark:text-red-400"
                  : "text-foreground"
              }
            >
              {role.is_superuser ? "Yes" : "No"}
            </span>
          </div>
          <div className="flex justify-between">
            <span className="text-muted-foreground">Create Database</span>
            <span className="text-foreground">
              {role.can_create_db ? "Yes" : "No"}
            </span>
          </div>
          <div className="flex justify-between">
            <span className="text-muted-foreground">Create Role</span>
            <span className="text-foreground">
              {role.can_create_role ? "Yes" : "No"}
            </span>
          </div>
          <div className="flex justify-between">
            <span className="text-muted-foreground">Connection Limit</span>
            <span className="text-foreground font-mono">
              {formatConnectionLimit(role.connection_limit)}
            </span>
          </div>
          <div className="flex justify-between">
            <span className="text-muted-foreground">Valid Until</span>
            <span className="text-foreground">
              {formatValidUntil(role.valid_until)}
            </span>
          </div>
        </div>
      </div>

      {/* Member Of */}
      {role.member_of.length > 0 && (
        <div>
          <h4 className="text-sm font-medium text-foreground mb-3">
            Member Of
          </h4>
          <div className="flex flex-wrap gap-2">
            {role.member_of.map((group) => (
              <span
                key={group}
                className="inline-flex items-center px-2 py-1 rounded text-xs font-medium bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400"
              >
                {group}
              </span>
            ))}
          </div>
        </div>
      )}

      {/* Table Permissions */}
      <div>
        <h4 className="text-sm font-medium text-foreground mb-3">
          Table Permissions
          {privileges.length > 0 && (
            <span className="ml-2 text-muted-foreground font-normal">
              ({privileges.length} table{privileges.length !== 1 ? "s" : ""})
            </span>
          )}
        </h4>
        {role.is_superuser ? (
          <p className="text-sm text-muted-foreground italic">
            Superuser has full access to all tables
          </p>
        ) : privileges.length === 0 ? (
          <p className="text-sm text-muted-foreground italic">
            No direct table permissions
          </p>
        ) : (
          <div className="space-y-2 max-h-64 overflow-y-auto">
            {Object.entries(groupedPrivileges).map(([table, privs]) => (
              <div
                key={table}
                className="flex items-start justify-between gap-4 py-2 border-b border-border last:border-b-0"
              >
                <span className="text-sm font-mono text-foreground">
                  {table}
                </span>
                <div className="flex flex-wrap gap-1 justify-end">
                  {privs.map((priv) => (
                    <span
                      key={priv}
                      className={`inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium ${
                        priv === "SELECT"
                          ? "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400"
                          : priv === "INSERT"
                            ? "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400"
                            : priv === "UPDATE"
                              ? "bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400"
                              : priv === "DELETE"
                                ? "bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400"
                                : "bg-muted text-muted-foreground"
                      }`}
                    >
                      {priv}
                    </span>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

export function RolesPage() {
  const { roles, isConnected, getPrivilegesForRole } = useDbStore();

  // Sheet state for role details
  const [sheetOpen, setSheetOpen] = useState(false);
  const [selectedRole, setSelectedRole] = useState<string | null>(null);

  const handleRoleClick = (roleName: string) => {
    setSelectedRole(roleName);
    setSheetOpen(true);
  };

  // Separate users (can login) from groups (cannot login)
  const users = useMemo(() => roles.filter((r) => r.can_login), [roles]);
  const groups = useMemo(() => roles.filter((r) => !r.can_login), [roles]);

  // Build permission summaries for each role
  const permissionSummaries = useMemo(() => {
    const summaries: Record<
      string,
      {
        summary: string;
        type: "superuser" | "read-write" | "read-only" | "mixed" | "none";
      }
    > = {};
    for (const role of roles) {
      if (role.is_superuser) {
        summaries[role.role_name] = { summary: "superuser", type: "superuser" };
      } else {
        const privileges = getPrivilegesForRole(role.role_name);
        summaries[role.role_name] = getPermissionSummary(privileges);
      }
    }
    return summaries;
  }, [roles, getPrivilegesForRole]);

  if (!isConnected) {
    return (
      <div className="flex-1 h-full flex items-center justify-center">
        <div className="text-center text-muted-foreground">
          <Users className="w-12 h-12 mx-auto mb-4 opacity-50" />
          <p>Connect to a database to view roles</p>
        </div>
      </div>
    );
  }

  if (roles.length === 0) {
    return (
      <div className="flex-1 h-full flex items-center justify-center">
        <div className="text-center text-muted-foreground">
          <Users className="w-12 h-12 mx-auto mb-4 opacity-50" />
          <p>No roles found</p>
        </div>
      </div>
    );
  }

  const getPermissionBadgeClass = (type: string) => {
    switch (type) {
      case "superuser":
        return "bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400";
      case "read-write":
        return "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400";
      case "read-only":
        return "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400";
      case "none":
        return "bg-muted text-muted-foreground";
      default:
        return "bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400";
    }
  };

  return (
    <div className="flex-1 h-full flex flex-col min-h-0">
      {/* Header */}
      <div className="flex-shrink-0 px-6 pt-6 pb-4 border-b border-border">
        <div className="flex items-center justify-between">
          <h2 className="text-2xl font-semibold text-foreground">Roles</h2>
          <div className="text-sm text-muted-foreground">
            {users.length} user{users.length !== 1 ? "s" : ""}, {groups.length}{" "}
            group{groups.length !== 1 ? "s" : ""}
          </div>
        </div>
      </div>

      {/* Scrollable Content */}
      <div className="flex-1 overflow-auto min-h-0 px-6 py-6">
        <div className="space-y-8">
          {/* Users Section */}
          <div>
            <h3 className="text-lg font-medium text-foreground mb-4">
              Users
              <span className="ml-2 text-sm font-normal text-muted-foreground">
                (can login)
              </span>
            </h3>
            {users.length === 0 ? (
              <p className="text-sm text-muted-foreground italic">
                No login users found
              </p>
            ) : (
              <Table containerClassName="overflow-x-auto">
                <TableHeader className="sticky top-0 z-10 bg-background">
                  <TableRow className="hover:bg-transparent">
                    <TableHead className="px-4 py-3 text-xs font-medium uppercase tracking-wider whitespace-nowrap bg-background">
                      User Name
                    </TableHead>
                    <TableHead className="px-4 py-3 text-xs font-medium uppercase tracking-wider whitespace-nowrap bg-background">
                      Permissions
                    </TableHead>
                    <TableHead className="px-4 py-3 text-xs font-medium uppercase tracking-wider whitespace-nowrap bg-background">
                      Member Of
                    </TableHead>
                    <TableHead className="px-4 py-3 text-xs font-medium uppercase tracking-wider whitespace-nowrap bg-background text-right">
                      Conn Limit
                    </TableHead>
                    <TableHead className="px-4 py-3 text-xs font-medium uppercase tracking-wider whitespace-nowrap bg-background text-right">
                      Valid Until
                    </TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {users.map((role) => {
                    const permSummary = permissionSummaries[role.role_name];
                    return (
                      <TableRow key={role.role_name}>
                        <TableCell className="px-4 py-3 whitespace-nowrap">
                          <button
                            onClick={() => handleRoleClick(role.role_name)}
                            className="font-mono text-blue-500 hover:text-blue-600 hover:underline cursor-pointer text-left"
                          >
                            {role.role_name}
                          </button>
                          {role.is_superuser && (
                            <span className="ml-2 inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400">
                              admin
                            </span>
                          )}
                        </TableCell>
                        <TableCell className="px-4 py-3 whitespace-nowrap">
                          <span
                            className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${getPermissionBadgeClass(permSummary.type)}`}
                          >
                            {permSummary.summary}
                          </span>
                        </TableCell>
                        <TableCell className="px-4 py-3 whitespace-nowrap">
                          {role.member_of.length > 0 ? (
                            <div className="flex flex-wrap gap-1">
                              {role.member_of.map((group) => (
                                <span
                                  key={group}
                                  className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-muted text-muted-foreground"
                                >
                                  {group}
                                </span>
                              ))}
                            </div>
                          ) : (
                            <span className="text-muted-foreground">—</span>
                          )}
                        </TableCell>
                        <TableCell className="px-4 py-3 whitespace-nowrap text-right font-mono">
                          {formatConnectionLimit(role.connection_limit)}
                        </TableCell>
                        <TableCell className="px-4 py-3 whitespace-nowrap text-right">
                          {formatValidUntil(role.valid_until)}
                        </TableCell>
                      </TableRow>
                    );
                  })}
                </TableBody>
              </Table>
            )}
          </div>

          {/* Groups Section */}
          <div>
            <h3 className="text-lg font-medium text-foreground mb-4">
              Groups
              <span className="ml-2 text-sm font-normal text-muted-foreground">
                (cannot login, used for permission inheritance)
              </span>
            </h3>
            {groups.length === 0 ? (
              <p className="text-sm text-muted-foreground italic">
                No group roles found
              </p>
            ) : (
              <Table containerClassName="overflow-x-auto">
                <TableHeader className="sticky top-0 z-10 bg-background">
                  <TableRow className="hover:bg-transparent">
                    <TableHead className="px-4 py-3 text-xs font-medium uppercase tracking-wider whitespace-nowrap bg-background">
                      Group Name
                    </TableHead>
                    <TableHead className="px-4 py-3 text-xs font-medium uppercase tracking-wider whitespace-nowrap bg-background">
                      Permissions
                    </TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {groups.map((role) => {
                    const permSummary = permissionSummaries[role.role_name];
                    return (
                      <TableRow key={role.role_name}>
                        <TableCell className="px-4 py-3 whitespace-nowrap">
                          <button
                            onClick={() => handleRoleClick(role.role_name)}
                            className="font-mono text-blue-500 hover:text-blue-600 hover:underline cursor-pointer text-left"
                          >
                            {role.role_name}
                          </button>
                        </TableCell>
                        <TableCell className="px-4 py-3 whitespace-nowrap">
                          <span
                            className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${getPermissionBadgeClass(permSummary.type)}`}
                          >
                            {permSummary.summary}
                          </span>
                        </TableCell>
                      </TableRow>
                    );
                  })}
                </TableBody>
              </Table>
            )}
          </div>
        </div>
      </div>

      {/* Role Details Sheet */}
      <Sheet open={sheetOpen} onOpenChange={setSheetOpen}>
        <SheetContent>
          <SheetClose />
          <SheetHeader>
            <SheetTitle className="font-mono">{selectedRole}</SheetTitle>
            <SheetDescription>Role details and permissions</SheetDescription>
          </SheetHeader>
          <SheetBody>
            <RoleDetails roleName={selectedRole} isLoading={false} />
          </SheetBody>
        </SheetContent>
      </Sheet>
    </div>
  );
}

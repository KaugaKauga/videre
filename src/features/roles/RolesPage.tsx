import { useDbStore } from "@/store/dbStore";
import { Users } from "@/components/icons";

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

export function RolesPage() {
  const { roles, isConnected } = useDbStore();

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

  const loginRoles = roles.filter((r) => r.can_login);
  const groupRoles = roles.filter((r) => !r.can_login);

  return (
    <div className="flex-1 h-full p-6 overflow-auto min-h-0">
      <div className="max-w-6xl mx-auto">
        <div className="mb-6">
          <h2 className="text-2xl font-semibold text-foreground mb-2">Roles</h2>
          <p className="text-sm text-muted-foreground">
            {loginRoles.length} login role{loginRoles.length !== 1 ? "s" : ""},{" "}
            {groupRoles.length} group role{groupRoles.length !== 1 ? "s" : ""}
          </p>
        </div>

        <div className="bg-card border border-border rounded-lg overflow-hidden">
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-border bg-muted/50">
                  <th className="text-left px-4 py-3 text-sm font-medium text-muted-foreground">
                    Role Name
                  </th>
                  <th className="text-center px-4 py-3 text-sm font-medium text-muted-foreground">
                    Login
                  </th>
                  <th className="text-center px-4 py-3 text-sm font-medium text-muted-foreground">
                    Superuser
                  </th>
                  <th className="text-center px-4 py-3 text-sm font-medium text-muted-foreground">
                    Create DB
                  </th>
                  <th className="text-center px-4 py-3 text-sm font-medium text-muted-foreground">
                    Create Role
                  </th>
                  <th className="text-right px-4 py-3 text-sm font-medium text-muted-foreground">
                    Conn Limit
                  </th>
                  <th className="text-left px-4 py-3 text-sm font-medium text-muted-foreground">
                    Member Of
                  </th>
                  <th className="text-right px-4 py-3 text-sm font-medium text-muted-foreground">
                    Valid Until
                  </th>
                </tr>
              </thead>
              <tbody>
                {roles.map((role, i) => (
                  <tr
                    key={role.role_name}
                    className={`border-b border-border last:border-b-0 ${
                      i % 2 === 0 ? "bg-background" : "bg-muted/20"
                    }`}
                  >
                    <td className="px-4 py-3 text-sm text-foreground font-mono">
                      <div className="flex items-center gap-2">
                        {role.role_name}
                        {role.is_superuser && (
                          <span className="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400">
                            admin
                          </span>
                        )}
                        {!role.can_login && (
                          <span className="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400">
                            group
                          </span>
                        )}
                      </div>
                    </td>
                    <td className="px-4 py-3 text-sm text-center">
                      {role.can_login ? (
                        <span className="text-green-600 dark:text-green-400">
                          ✓
                        </span>
                      ) : (
                        <span className="text-muted-foreground">—</span>
                      )}
                    </td>
                    <td className="px-4 py-3 text-sm text-center">
                      {role.is_superuser ? (
                        <span className="text-red-600 dark:text-red-400">
                          ✓
                        </span>
                      ) : (
                        <span className="text-muted-foreground">—</span>
                      )}
                    </td>
                    <td className="px-4 py-3 text-sm text-center">
                      {role.can_create_db ? (
                        <span className="text-green-600 dark:text-green-400">
                          ✓
                        </span>
                      ) : (
                        <span className="text-muted-foreground">—</span>
                      )}
                    </td>
                    <td className="px-4 py-3 text-sm text-center">
                      {role.can_create_role ? (
                        <span className="text-green-600 dark:text-green-400">
                          ✓
                        </span>
                      ) : (
                        <span className="text-muted-foreground">—</span>
                      )}
                    </td>
                    <td className="px-4 py-3 text-sm text-foreground text-right font-mono">
                      {formatConnectionLimit(role.connection_limit)}
                    </td>
                    <td className="px-4 py-3 text-sm text-foreground">
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
                    </td>
                    <td className="px-4 py-3 text-sm text-foreground text-right">
                      {formatValidUntil(role.valid_until)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </div>
  );
}

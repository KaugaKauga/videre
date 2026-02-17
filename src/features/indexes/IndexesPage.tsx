import { useMemo } from "react";
import { useDbStore } from "@/store/dbStore";
import { ListTree } from "@/components/icons";

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

export function IndexesPage() {
  const { tables, indexes } = useDbStore();

  // Flatten all indexes into a single array
  const allIndexes = useMemo(() => {
    const result: Array<{
      index_name: string;
      table_schema: string;
      table_name: string;
      columns: string[];
      is_unique: boolean;
      is_primary: boolean;
      index_type: string;
      size_bytes: number;
    }> = [];

    for (const table of tables) {
      const key = `${table.schema}.${table.name}`;
      const tableIndexes = indexes[key] || [];
      result.push(...tableIndexes);
    }

    // Sort by schema, table, then index name
    return result.sort((a, b) => {
      const schemaCompare = a.table_schema.localeCompare(b.table_schema);
      if (schemaCompare !== 0) return schemaCompare;
      const tableCompare = a.table_name.localeCompare(b.table_name);
      if (tableCompare !== 0) return tableCompare;
      return a.index_name.localeCompare(b.index_name);
    });
  }, [tables, indexes]);

  if (allIndexes.length === 0) {
    return (
      <div className="flex-1 h-full flex items-center justify-center">
        <div className="text-center text-muted-foreground">
          <ListTree className="w-12 h-12 mx-auto mb-4 opacity-50" />
          <p>No indexes found</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 h-full p-6 overflow-auto min-h-0">
      <div className="max-w-6xl mx-auto">
        <div className="mb-6">
          <h2 className="text-2xl font-semibold text-foreground mb-2">
            Indexes
          </h2>
          <p className="text-sm text-muted-foreground">
            {allIndexes.length} index{allIndexes.length !== 1 ? "es" : ""}{" "}
            across {tables.length} table{tables.length !== 1 ? "s" : ""}
          </p>
        </div>

        <div className="bg-card border border-border rounded-lg overflow-hidden">
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-border bg-muted/50">
                  <th className="text-left px-4 py-3 text-sm font-medium text-muted-foreground">
                    Index Name
                  </th>
                  <th className="text-left px-4 py-3 text-sm font-medium text-muted-foreground">
                    Table
                  </th>
                  <th className="text-left px-4 py-3 text-sm font-medium text-muted-foreground">
                    Columns
                  </th>
                  <th className="text-left px-4 py-3 text-sm font-medium text-muted-foreground">
                    Type
                  </th>
                  <th className="text-center px-4 py-3 text-sm font-medium text-muted-foreground">
                    Unique
                  </th>
                  <th className="text-center px-4 py-3 text-sm font-medium text-muted-foreground">
                    Primary
                  </th>
                  <th className="text-right px-4 py-3 text-sm font-medium text-muted-foreground">
                    Size
                  </th>
                </tr>
              </thead>
              <tbody>
                {allIndexes.map((index, i) => (
                  <tr
                    key={`${index.table_schema}.${index.table_name}.${index.index_name}`}
                    className={`border-b border-border last:border-b-0 ${
                      i % 2 === 0 ? "bg-background" : "bg-muted/20"
                    }`}
                  >
                    <td className="px-4 py-3 text-sm text-foreground font-mono">
                      {index.index_name}
                    </td>
                    <td className="px-4 py-3 text-sm text-foreground">
                      <span className="text-muted-foreground">
                        {index.table_schema}.
                      </span>
                      {index.table_name}
                    </td>
                    <td className="px-4 py-3 text-sm text-foreground font-mono">
                      {index.columns.join(", ")}
                    </td>
                    <td className="px-4 py-3 text-sm text-foreground">
                      <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-muted text-muted-foreground">
                        {index.index_type}
                      </span>
                    </td>
                    <td className="px-4 py-3 text-sm text-center">
                      {index.is_unique ? (
                        <span className="text-green-600 dark:text-green-400">
                          ✓
                        </span>
                      ) : (
                        <span className="text-muted-foreground">—</span>
                      )}
                    </td>
                    <td className="px-4 py-3 text-sm text-center">
                      {index.is_primary ? (
                        <span className="text-blue-600 dark:text-blue-400">
                          ✓
                        </span>
                      ) : (
                        <span className="text-muted-foreground">—</span>
                      )}
                    </td>
                    <td className="px-4 py-3 text-sm text-foreground text-right font-mono">
                      {formatBytes(index.size_bytes)}
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

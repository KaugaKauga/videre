import { useMemo } from "react";
import { useDbStore } from "@/store/dbStore";
import { ListTree } from "@/components/icons";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

export function IndexesPage() {
  const { tables, indexes, isConnected } = useDbStore();

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

  if (!isConnected) {
    return (
      <div className="flex-1 h-full flex items-center justify-center">
        <div className="text-center text-muted-foreground">
          <ListTree className="w-12 h-12 mx-auto mb-4 opacity-50" />
          <p>Connect to a database to view indexes</p>
        </div>
      </div>
    );
  }

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
    <div className="flex-1 h-full flex flex-col min-h-0">
      {/* Header */}
      <div className="flex-shrink-0 px-6 pt-6 pb-4 border-b border-border">
        <div className="flex items-center justify-between">
          <h2 className="text-2xl font-semibold text-foreground">Indexes</h2>
          <div className="text-sm text-muted-foreground">
            {allIndexes.length} index{allIndexes.length !== 1 ? "es" : ""}{" "}
            across {tables.length} table{tables.length !== 1 ? "s" : ""}
          </div>
        </div>
      </div>

      {/* Scrollable Table */}
      <div className="flex-1 overflow-auto min-h-0 px-6 py-6">
        <div className="h-full flex flex-col">
          <Table containerClassName="overflow-x-auto">
            <TableHeader className="sticky top-0 z-10 bg-background">
              <TableRow className="hover:bg-transparent">
                <TableHead className="px-4 py-3 text-xs font-medium uppercase tracking-wider whitespace-nowrap bg-background">
                  Index Name
                </TableHead>
                <TableHead className="px-4 py-3 text-xs font-medium uppercase tracking-wider whitespace-nowrap bg-background">
                  Table
                </TableHead>
                <TableHead className="px-4 py-3 text-xs font-medium uppercase tracking-wider whitespace-nowrap bg-background">
                  Columns
                </TableHead>
                <TableHead className="px-4 py-3 text-xs font-medium uppercase tracking-wider whitespace-nowrap bg-background">
                  Type
                </TableHead>
                <TableHead className="px-4 py-3 text-xs font-medium uppercase tracking-wider whitespace-nowrap bg-background text-center">
                  Unique
                </TableHead>
                <TableHead className="px-4 py-3 text-xs font-medium uppercase tracking-wider whitespace-nowrap bg-background text-center">
                  Primary
                </TableHead>
                <TableHead className="px-4 py-3 text-xs font-medium uppercase tracking-wider whitespace-nowrap bg-background text-right">
                  Size
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {allIndexes.map((index) => (
                <TableRow
                  key={`${index.table_schema}.${index.table_name}.${index.index_name}`}
                >
                  <TableCell className="px-4 py-3 whitespace-nowrap font-mono">
                    {index.index_name}
                  </TableCell>
                  <TableCell className="px-4 py-3 whitespace-nowrap">
                    <span className="text-muted-foreground">
                      {index.table_schema}.
                    </span>
                    {index.table_name}
                  </TableCell>
                  <TableCell className="px-4 py-3 whitespace-nowrap font-mono">
                    {index.columns.join(", ")}
                  </TableCell>
                  <TableCell className="px-4 py-3 whitespace-nowrap">
                    <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-muted text-muted-foreground">
                      {index.index_type}
                    </span>
                  </TableCell>
                  <TableCell className="px-4 py-3 whitespace-nowrap text-center">
                    {index.is_unique ? (
                      <span className="text-green-600 dark:text-green-400">
                        ✓
                      </span>
                    ) : (
                      <span className="text-muted-foreground">—</span>
                    )}
                  </TableCell>
                  <TableCell className="px-4 py-3 whitespace-nowrap text-center">
                    {index.is_primary ? (
                      <span className="text-blue-600 dark:text-blue-400">
                        ✓
                      </span>
                    ) : (
                      <span className="text-muted-foreground">—</span>
                    )}
                  </TableCell>
                  <TableCell className="px-4 py-3 whitespace-nowrap text-right font-mono">
                    {formatBytes(index.size_bytes)}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      </div>
    </div>
  );
}

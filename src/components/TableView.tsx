import { useEffect, useState } from "react";
import { Loader2, ChevronLeft, ChevronRight } from "lucide-react";
import { Button } from "@/components/ui/button";
import { db, TableData } from "@/lib/tauri";

interface TableViewProps {
  tableName: string;
  schema?: string;
}

export function TableView({ tableName, schema = "public" }: TableViewProps) {
  const [data, setData] = useState<TableData | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [page, setPage] = useState(0);
  const [limit] = useState(100);

  useEffect(() => {
    const fetchData = async () => {
      setIsLoading(true);
      setError(null);
      try {
        const result = await db.getTableData(
          tableName,
          schema,
          limit,
          page * limit,
        );
        setData(result);
      } catch (err) {
        setError(`Failed to fetch data: ${err}`);
      } finally {
        setIsLoading(false);
      }
    };

    fetchData();
  }, [tableName, schema, page, limit]);

  const totalPages = data ? Math.ceil(data.total_rows / limit) : 0;
  const hasNextPage = page < totalPages - 1;
  const hasPrevPage = page > 0;

  if (isLoading) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <div className="flex items-center gap-2 text-muted-foreground">
          <Loader2 className="w-5 h-5 animate-spin" />
          <span>Loading data...</span>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex-1 p-6">
        <div className="max-w-7xl mx-auto">
          <div className="bg-red-500/10 border border-red-500/20 rounded-lg p-4">
            <p className="text-red-600 dark:text-red-400">{error}</p>
          </div>
        </div>
      </div>
    );
  }

  if (!data || data.rows.length === 0) {
    return (
      <div className="flex-1 p-6">
        <div className="max-w-7xl mx-auto">
          <h2 className="text-2xl font-semibold text-foreground mb-4 capitalize">
            {tableName}
          </h2>
          <div className="bg-card border border-border rounded-lg p-8 text-center">
            <p className="text-muted-foreground">No data found in this table</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <div className="p-6 pb-4">
        <div className="max-w-7xl mx-auto">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-2xl font-semibold text-foreground capitalize">
              {tableName}
            </h2>
            <div className="text-sm text-muted-foreground">
              {data.total_rows.toLocaleString()} total rows
            </div>
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-auto px-6">
        <div className="max-w-7xl mx-auto">
          <div className="bg-card border border-border rounded-lg overflow-hidden">
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead>
                  <tr className="border-b border-border bg-muted/50">
                    {data.columns.map((column) => (
                      <th
                        key={column}
                        className="px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider whitespace-nowrap"
                      >
                        {column}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody className="divide-y divide-border">
                  {data.rows.map((row, rowIndex) => (
                    <tr
                      key={rowIndex}
                      className="hover:bg-accent/50 transition-colors"
                    >
                      {row.map((cell, cellIndex) => (
                        <td
                          key={cellIndex}
                          className="px-4 py-3 text-sm text-foreground whitespace-nowrap"
                        >
                          {cell === null ? (
                            <span className="text-muted-foreground italic">
                              NULL
                            </span>
                          ) : typeof cell === "object" ? (
                            <span className="text-muted-foreground">
                              {JSON.stringify(cell)}
                            </span>
                          ) : (
                            String(cell)
                          )}
                        </td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      </div>

      <div className="p-6 pt-4">
        <div className="max-w-7xl mx-auto">
          <div className="flex items-center justify-between">
            <div className="text-sm text-muted-foreground">
              Showing {page * limit + 1} to{" "}
              {Math.min((page + 1) * limit, data.total_rows)} of{" "}
              {data.total_rows.toLocaleString()} rows
            </div>
            <div className="flex items-center gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => setPage((p) => p - 1)}
                disabled={!hasPrevPage}
              >
                <ChevronLeft className="w-4 h-4 mr-1" />
                Previous
              </Button>
              <div className="text-sm text-muted-foreground">
                Page {page + 1} of {totalPages}
              </div>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setPage((p) => p + 1)}
                disabled={!hasNextPage}
              >
                Next
                <ChevronRight className="w-4 h-4 ml-1" />
              </Button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

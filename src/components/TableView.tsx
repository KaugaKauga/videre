import { useEffect, useState } from "react";
import { Loader2, ChevronLeft, ChevronRight } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
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
        console.log(
          `Fetching data for table: ${schema}.${tableName} (page ${page + 1}, limit ${limit})...`,
        );
        const result = await db.getTableData(
          tableName,
          schema,
          limit,
          page * limit,
        );
        console.log(
          `Loaded ${result.rows.length} rows from ${schema}.${tableName} (${result.total_rows} total rows)`,
        );
        setData(result);
      } catch (err) {
        console.error(`Failed to fetch data for ${schema}.${tableName}:`, err);
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
    <div className="flex-1 h-full flex flex-col min-h-0">
      {/* Header */}
      <div className="flex-shrink-0 px-6 pt-6 pb-4 border-b border-border">
        <div className="max-w-7xl mx-auto">
          <div className="flex items-center justify-between">
            <h2 className="text-2xl font-semibold text-foreground capitalize">
              {tableName}
            </h2>
            <div className="text-sm text-muted-foreground">
              {data.total_rows.toLocaleString()} total rows
            </div>
          </div>
        </div>
      </div>

      {/* Scrollable Table */}
      <div className="flex-1 overflow-auto min-h-0 px-6 py-6">
        <div className="max-w-7xl mx-auto">
          <div className="bg-card border border-border rounded-lg overflow-hidden">
            <Table containerClassName="overflow-x-auto">
              <TableHeader className="sticky top-0 z-10 bg-muted/50">
                <TableRow className="hover:bg-transparent">
                  {data.columns.map((column) => (
                    <TableHead
                      key={column}
                      className="px-4 py-3 text-xs font-medium uppercase tracking-wider whitespace-nowrap bg-muted/50"
                    >
                      {column}
                    </TableHead>
                  ))}
                </TableRow>
              </TableHeader>
              <TableBody>
                {data.rows.map((row, rowIndex) => (
                  <TableRow key={rowIndex}>
                    {row.map((cell, cellIndex) => (
                      <TableCell
                        key={cellIndex}
                        className="px-4 py-3 whitespace-nowrap"
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
                      </TableCell>
                    ))}
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>
        </div>
      </div>

      {/* Footer - Sticky at bottom */}
      <div className="flex-shrink-0 px-6 py-4 border-t border-border bg-background">
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

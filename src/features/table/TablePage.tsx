import { useEffect, useState, useMemo } from "react";
import {
  Loader,
  ChevronLeft,
  ChevronRight,
  ArrowUpDown,
} from "@/components/icons";
import { Button } from "@/components/ui/button";
import { DataTable } from "../../components/DataTable";
import { db, TableData, ForeignKeyInfo, RowData } from "@/lib/tauri";
import { useDbStore } from "@/store/dbStore";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetDescription,
  SheetBody,
  SheetClose,
} from "@/components/ui/sheet";

interface TableViewProps {
  tableName: string;
  schema?: string;
}

interface FkCellInfo {
  foreignTableSchema: string;
  foreignTableName: string;
  foreignColumnName: string;
}

// Component to display the referenced row data in the sheet
function RowDetails({
  data,
  isLoading,
  error,
}: {
  data: RowData | null;
  isLoading: boolean;
  error: string | null;
}) {
  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-8">
        <Loader className="w-5 h-5 animate-spin mr-2" />
        <span className="text-muted-foreground">Loading...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-500/10 border border-red-500/20 rounded-lg p-4">
        <p className="text-red-600 dark:text-red-400 text-sm">{error}</p>
      </div>
    );
  }

  if (!data) {
    return <p className="text-muted-foreground text-sm">No data to display</p>;
  }

  return (
    <div className="space-y-2">
      {data.columns.map((column, index) => (
        <div key={column} className="flex gap-2">
          <span className="text-sm font-medium text-muted-foreground min-w-[120px] shrink-0">
            {column}:
          </span>
          <span className="text-sm text-foreground break-all">
            {data.values[index] === null ? (
              <span className="text-muted-foreground italic">NULL</span>
            ) : typeof data.values[index] === "object" ? (
              <code className="text-xs bg-muted px-2 py-1 rounded">
                {JSON.stringify(data.values[index], null, 2)}
              </code>
            ) : (
              String(data.values[index])
            )}
          </span>
        </div>
      ))}
    </div>
  );
}

// Helper component to render the table content with FK-aware cells
function DataTableContent({
  data,
  foreignKeys,
  onFkClick,
}: {
  data: TableData;
  foreignKeys: ForeignKeyInfo[];
  onFkClick: (fkInfo: FkCellInfo, value: any) => void;
}) {
  // Build a map of column name -> FK info for quick lookup
  const fkMap = useMemo(() => {
    const map = new Map<string, ForeignKeyInfo>();
    for (const fk of foreignKeys) {
      map.set(fk.column_name, fk);
    }
    return map;
  }, [foreignKeys]);

  // Create columns dynamically from the data
  const columns = useMemo(() => {
    return data.columns.map((columnName, index) => {
      const fkInfo = fkMap.get(columnName);

      return {
        accessorFn: (row: any[]) => row[index],
        id: columnName,
        header: ({
          toggleSort,
        }: {
          toggleSort: () => void;
          sortDirection: "asc" | "desc" | null;
        }) => {
          return (
            <Button
              variant="ghost"
              onClick={toggleSort}
              className="h-auto p-0 hover:bg-transparent"
            >
              {columnName}
              {fkInfo && (
                <span
                  className="ml-1 text-xs text-blue-500"
                  title={`FK → ${fkInfo.foreign_table_name}`}
                >
                  🔗
                </span>
              )}
              <ArrowUpDown className="ml-2 h-3 w-3" />
            </Button>
          );
        },
        cell: ({ value }: { value: any }) => {
          if (value === null) {
            return <span className="text-muted-foreground italic">NULL</span>;
          }

          // If this is a FK column, make it clickable
          if (fkInfo) {
            return (
              <button
                onClick={() =>
                  onFkClick(
                    {
                      foreignTableSchema: fkInfo.foreign_table_schema,
                      foreignTableName: fkInfo.foreign_table_name,
                      foreignColumnName: fkInfo.foreign_column_name,
                    },
                    value,
                  )
                }
                className="text-blue-500 hover:text-blue-600 hover:underline cursor-pointer text-left"
                title={`View ${fkInfo.foreign_table_name} record`}
              >
                {String(value)}
              </button>
            );
          }

          if (typeof value === "object") {
            return (
              <span className="text-muted-foreground">
                {JSON.stringify(value)}
              </span>
            );
          }
          return String(value);
        },
      };
    });
  }, [data.columns, fkMap, onFkClick]);

  return <DataTable columns={columns} data={data.rows} />;
}

export function TablePage({ tableName, schema = "public" }: TableViewProps) {
  const [data, setData] = useState<TableData | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [page, setPage] = useState(0);
  const [limit] = useState(100);

  // Sheet state for FK row preview
  const [sheetOpen, setSheetOpen] = useState(false);
  const [sheetData, setSheetData] = useState<RowData | null>(null);
  const [sheetLoading, setSheetLoading] = useState(false);
  const [sheetError, setSheetError] = useState<string | null>(null);
  const [sheetTitle, setSheetTitle] = useState("");

  // Get FK info from store
  const getForeignKeysForTable = useDbStore(
    (state) => state.getForeignKeysForTable,
  );
  const foreignKeys = getForeignKeysForTable(tableName, schema);

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

  // Handle FK cell click - fetch the referenced row and open sheet
  const handleFkClick = async (fkInfo: FkCellInfo, value: any) => {
    setSheetTitle(`${fkInfo.foreignTableName}`);
    setSheetOpen(true);
    setSheetLoading(true);
    setSheetError(null);
    setSheetData(null);

    try {
      const rowData = await db.getRowByPk(
        fkInfo.foreignTableName,
        fkInfo.foreignTableSchema,
        fkInfo.foreignColumnName,
        value,
      );
      setSheetData(rowData);
    } catch (err) {
      console.error("Failed to fetch referenced row:", err);
      setSheetError(`Failed to fetch row: ${err}`);
    } finally {
      setSheetLoading(false);
    }
  };

  const totalPages = data ? Math.ceil(data.total_rows / limit) : 0;
  const hasNextPage = page < totalPages - 1;
  const hasPrevPage = page > 0;

  if (isLoading) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <div className="flex items-center gap-2 text-muted-foreground">
          <Loader className="w-5 h-5 animate-spin" />
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

  if (!data) {
    return (
      <div className="flex-1 p-6">
        <div className="max-w-7xl mx-auto">
          <h2 className="text-2xl font-semibold text-foreground mb-4 capitalize">
            {tableName}
          </h2>
          <div className="bg-card border border-border rounded-lg p-8 text-center">
            <p className="text-muted-foreground">No data available</p>
          </div>
        </div>
      </div>
    );
  }

  const isEmpty = data.rows.length === 0;

  return (
    <div className="flex-1 h-full flex flex-col min-h-0">
      {/* Header */}
      <div className="flex-shrink-0 px-6 pt-6 pb-4 border-b border-border">
        <div className="flex items-center justify-between">
          <h2 className="text-2xl font-semibold text-foreground capitalize">
            {tableName}
          </h2>
          <div className="text-sm text-muted-foreground">
            {isEmpty ? (
              <span>Empty table • {data.columns.length} columns</span>
            ) : (
              <span>{data.total_rows.toLocaleString()} total rows</span>
            )}
          </div>
        </div>
      </div>

      {/* Scrollable Table */}
      <div className="flex-1 overflow-auto min-h-0 px-6 py-6">
        <div className="h-full flex flex-col">
          <DataTableContent
            data={data}
            foreignKeys={foreignKeys}
            onFkClick={handleFkClick}
          />
          {isEmpty && (
            <div className="flex-1 flex items-center justify-center py-12">
              <p className="text-muted-foreground">No rows in this table</p>
            </div>
          )}
        </div>
      </div>

      {/* Footer - Sticky at bottom */}
      {!isEmpty && (
        <div className="flex-shrink-0 px-6 py-4 border-t border-border bg-background">
          <div className="flex items-center justify-between">
            <div className="text-sm text-muted-foreground">
              Showing {page * limit + 1} to{" "}
              {Math.min((page + 1) * limit, data.total_rows)} of{" "}
              {data.total_rows.toLocaleString()} rows
            </div>
            <div className="flex items-center gap-2">
              <Button
                variant="ghost"
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
                variant="ghost"
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
      )}

      {/* FK Row Preview Sheet */}
      <Sheet open={sheetOpen} onOpenChange={setSheetOpen}>
        <SheetContent>
          <SheetClose />
          <SheetHeader>
            <SheetTitle>{sheetTitle}</SheetTitle>
            <SheetDescription>Referenced row details</SheetDescription>
          </SheetHeader>
          <SheetBody>
            <RowDetails
              data={sheetData}
              isLoading={sheetLoading}
              error={sheetError}
            />
          </SheetBody>
        </SheetContent>
      </Sheet>
    </div>
  );
}

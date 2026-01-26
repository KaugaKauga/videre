import { useState } from "react";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

interface Column {
  id: string;
  header: (props: {
    toggleSort: () => void;
    sortDirection: "asc" | "desc" | null;
  }) => React.ReactNode;
  cell: (props: { value: any }) => React.ReactNode;
  accessorFn: (row: any) => any;
}

interface DataTableProps {
  columns: Column[];
  data: any[];
}

export function DataTable({ columns, data }: DataTableProps) {
  const [sorting, setSorting] = useState<{
    columnId: string;
    direction: "asc" | "desc";
  } | null>(null);

  const toggleSort = (columnId: string) => {
    setSorting((prev) => {
      if (prev?.columnId === columnId) {
        // Same column: toggle direction or reset
        if (prev.direction === "asc") {
          return { columnId, direction: "desc" };
        }
        return null; // Reset sorting
      }
      // New column: start with ascending
      return { columnId, direction: "asc" };
    });
  };

  const sortedData = [...data];
  if (sorting) {
    const column = columns.find((col) => col.id === sorting.columnId);
    if (column) {
      sortedData.sort((a, b) => {
        const aVal = column.accessorFn(a);
        const bVal = column.accessorFn(b);

        // Handle null values
        if (aVal === null && bVal === null) return 0;
        if (aVal === null) return 1;
        if (bVal === null) return -1;

        // Compare values
        let comparison = 0;
        if (typeof aVal === "string" && typeof bVal === "string") {
          comparison = aVal.localeCompare(bVal);
        } else if (typeof aVal === "number" && typeof bVal === "number") {
          comparison = aVal - bVal;
        } else {
          comparison = String(aVal).localeCompare(String(bVal));
        }

        return sorting.direction === "asc" ? comparison : -comparison;
      });
    }
  }

  return (
    <div className="flex flex-col h-full">
      <Table containerClassName="overflow-x-auto">
        <TableHeader className="sticky top-0 z-10 bg-muted/50">
          <TableRow className="hover:bg-transparent">
            {columns.map((column) => (
              <TableHead
                key={column.id}
                className="px-4 py-3 text-xs font-medium uppercase tracking-wider whitespace-nowrap bg-muted/50"
              >
                {column.header({
                  toggleSort: () => toggleSort(column.id),
                  sortDirection:
                    sorting?.columnId === column.id ? sorting.direction : null,
                })}
              </TableHead>
            ))}
          </TableRow>
        </TableHeader>
        <TableBody>
          {sortedData.length > 0 ? (
            sortedData.map((row, rowIndex) => (
              <TableRow key={rowIndex}>
                {columns.map((column) => (
                  <TableCell
                    key={column.id}
                    className="px-4 py-3 whitespace-nowrap"
                  >
                    {column.cell({ value: column.accessorFn(row) })}
                  </TableCell>
                ))}
              </TableRow>
            ))
          ) : (
            <TableRow>
              <TableCell colSpan={columns.length} className="h-24 text-center">
                No results.
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  );
}

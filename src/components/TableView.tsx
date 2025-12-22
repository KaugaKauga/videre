interface TableViewProps {
  tableName: string;
}

export function TableView({ tableName }: TableViewProps) {
  return (
    <div className="flex-1 p-6 overflow-auto">
      <div className="max-w-7xl mx-auto">
        <h2 className="text-2xl font-semibold text-foreground mb-4 capitalize">
          {tableName}
        </h2>
        <div className="bg-card border border-border rounded-lg overflow-hidden">
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-border bg-muted/50">
                  <th className="px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    ID
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    Name
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    Created At
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border">
                <tr className="hover:bg-accent/50 transition-colors">
                  <td className="px-4 py-3 text-sm text-foreground">1</td>
                  <td className="px-4 py-3 text-sm text-foreground">Sample Entry</td>
                  <td className="px-4 py-3 text-sm text-muted-foreground">2024-01-15</td>
                </tr>
                <tr className="hover:bg-accent/50 transition-colors">
                  <td className="px-4 py-3 text-sm text-foreground">2</td>
                  <td className="px-4 py-3 text-sm text-foreground">Another Entry</td>
                  <td className="px-4 py-3 text-sm text-muted-foreground">2024-01-16</td>
                </tr>
                <tr className="hover:bg-accent/50 transition-colors">
                  <td className="px-4 py-3 text-sm text-foreground">3</td>
                  <td className="px-4 py-3 text-sm text-foreground">Third Entry</td>
                  <td className="px-4 py-3 text-sm text-muted-foreground">2024-01-17</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
        <div className="mt-4 text-sm text-muted-foreground">
          Showing dummy data for <span className="font-medium text-foreground">{tableName}</span> table
        </div>
      </div>
    </div>
  );
}

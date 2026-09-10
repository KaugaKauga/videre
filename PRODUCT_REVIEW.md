# Videre Product Review

_Date: August 2026_

_Last progress update: September 10, 2026_

## Delivery progress

Current focus: **Reader interactions and relation structure**

- [x] Deterministic PostgreSQL ordering and pagination.
- [x] Server-side sorting across the complete relation.
- [x] Natural text selection and keyboard copying in data cells and referenced-row details.
- [ ] Relation and column information. **Next read-only slice**
- [ ] Refresh and reliable loading/error states.
- [ ] Explicit copy cell, copy row, and full-value inspection actions.
- [ ] Schema-qualified relation identity.
- [ ] Active connection identity and lifecycle controls.
- [ ] Lazy metadata loading.
- [ ] Server-side filtering. **Deferred until the query/filter model is designed deliberately.**

Completed reader work is covered by backend unit tests, frontend unit tests, and live PostgreSQL integration tests using the project test database. The sorting implementation uses native PostgreSQL ordering where supported, explicit null placement, deterministic primary-key tie-breakers, and a safe text fallback for types without native ordering.

## Executive verdict

Videre has a strong foundation for a focused PostgreSQL product. The current connection flow, desktop shell, tabs, compact data tables, and foreign-key drill-down form a coherent early experience.

The current implementation is best described as a **read-only PostgreSQL data explorer**. It is not yet a complete database inspector and does not currently provide operational observability.

The product is on the right track, but the next iterations should strengthen the primary reading and inspection workflow before adding editing, additional database engines, or more visual themes.

## Recommended product direction

### Phase 1 — Reader Mode

The initial and primary product should be a beautiful, simple, secure, and trustworthy PostgreSQL reader.

Reader Mode should be a first-class product mode rather than a temporary limitation. It should remain available after editing is introduced and should be the default mode for new connections.

Reader Mode should let a user:

- Connect quickly and understand which database is active.
- Find any PostgreSQL object without knowing its exact schema.
- Browse data accurately.
- Filter, sort, refresh, copy, and inspect complete values.
- Understand relation structure, constraints, indexes, and relationships.
- Follow relationships without writing SQL.
- Inspect live PostgreSQL activity and health without modifying the database.

The product can already be called a **database inspector** in this phase. Inspection usually means understanding data and structure; it does not require mutation capabilities.

### Phase 2 — Edit Mode

Once Reader Mode is excellent, Videre can add an explicit, opt-in Edit Mode with:

- Insert row.
- Update row.
- Delete row.
- Clear previews of pending changes.
- Explicit confirmation for destructive operations.
- A visually unmistakable indication that editing is enabled.

Reader Mode should remain the default and should never feel like a reduced version of the product.

Recommended mode names:

- **Reader Mode**
- **Edit Mode**

Avoid calling the second mode “Inspector Mode,” because the read-only product is already an inspector and the name would not communicate that mutations are enabled.

### Future database engines

MySQL, SQLite, and other database engines should come after the PostgreSQL experience is demonstrably excellent. PostgreSQL depth is currently a better differentiator than broad but shallow compatibility.

## What Videre currently does

The implemented product supports:

- PostgreSQL connection and connection testing.
- Recent connection profiles.
- A resizable sidebar and desktop tab model.
- A list of database relations.
- Deterministically ordered, paginated row browsing.
- Server-side sorting across the complete relation.
- Foreign-key markers and referenced-row drawers.
- A global index summary.
- Role and direct table-grant inspection.
- Multiple themes and configurable UI font size.

Relevant implementation areas:

- `src-leptos/src/pages/connection.rs`
- `src-leptos/src/components/sidebar.rs`
- `src-leptos/src/components/shell.rs`
- `src-leptos/src/pages/table.rs`
- `src-leptos/src/components/data_table.rs`
- `src-leptos/src/pages/indexes.rs`
- `src-leptos/src/pages/roles.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/pg/catalog.rs`
- `src-tauri/src/pg/data.rs`

## What is working well

### A clear read-only boundary

Read-only operation is a strong product decision. It lowers cognitive load, reduces the risk of accidental changes, and prevents Videre from immediately becoming another general-purpose database IDE.

### Appropriate desktop information architecture

The sidebar, object tabs, compact tables, and contextual drawers fit the way developers inspect databases. Reopening an existing table focuses its tab rather than duplicating it.

### Foreign-key exploration

Clickable foreign-key values opening referenced rows are the most promising differentiating workflow. This directly helps users trace a problem through related data without writing joins.

This should be expanded into a broader relationship-navigation experience.

### Visual ambition

The Swiss and Cassette Futurism themes provide a real visual identity. Videre does not look like a generic enterprise database client.

Future effort should favor interaction quality over additional themes. A top-notch UI must make database information accurate, legible, and easy to manipulate—not only visually distinctive.

### PostgreSQL value handling

The backend has a useful foundation for displaying a broad range of PostgreSQL values. Unsupported types fall back to PostgreSQL text output instead of silently appearing as null.

## Critical product findings

### 1. Positioning mismatch

There are three related product categories:

1. **Data reader** — browses and filters rows.
2. **Database inspector** — explains data, structure, constraints, relationships, indexes, and object definitions.
3. **Observability tool** — diagnoses live queries, locks, waits, storage, maintenance, and performance.

Videre currently implements the first category with parts of the second. It does not yet implement the third.

Until live diagnostic capabilities exist, the product should be positioned as a PostgreSQL reader, explorer, or inspector rather than a general observability tool.

A suitable near-term description is:

> A fast, local, read-only PostgreSQL explorer for understanding data.

A later description could be:

> A local PostgreSQL inspector for understanding data and diagnosing what is happening now.

### 2. Data browsing correctness

The primary table experience originally had several trust issues. Ordering and sorting are now resolved; exact counting remains open.

#### Unordered pagination — Resolved

Page fetches now always include deterministic ordering. Relations use their primary key, including composite keys, when available and fall back to PostgreSQL text ordering across displayed columns when no primary key exists.

#### Page-local sorting — Resolved

Column-header sorting now executes in PostgreSQL across the complete relation. Sort changes reset to the first page, persist during pagination, place nulls last, and use primary-key columns as deterministic tie-breakers. Unknown columns are rejected before SQL construction.

#### Expensive exact counts

An exact `COUNT(*)` is executed for every page fetch. This can make apparently simple browsing unexpectedly expensive on large tables.

These issues should be treated as product-correctness work, not optional performance polish.

### 3. Missing relation structure

A database reader should do more than display row values. It should explain what the relation is.

A relation detail view should include:

- Relation kind: table, view, materialized view, foreign table, or partitioned table.
- Columns and exact PostgreSQL types.
- Nullability.
- Defaults.
- Identity and generated properties.
- Primary and unique constraints.
- Foreign-key definitions and actions.
- Check and exclusion constraints.
- Index definitions.
- Comments.
- Table and index size.
- View definition where applicable.
- Trigger information.
- Partition parent and children where applicable.

Recommended relation tabs:

- **Data**
- **Structure**
- **Relationships**
- **Indexes**

Views can additionally expose **Definition**, and partitioned tables can expose **Partitions** where relevant.

#### Feature brief — Column information drawer

**Status:** Planned — next read-only implementation slice.

**Product outcome:** A user can understand what a column means and how PostgreSQL defines it without leaving the data they are reading, opening another tool, or writing catalog SQL.

##### Entry point and interaction

The action should live in each data-table column header because the column is already the user's visual context. Keep the existing header-label click for sorting and add a separate, compact information button beside the column name, FK badge, and sort indicator.

- Use a recognizable information icon with the tooltip and accessible label `Column information for <column>`.
- Keep the action in the DOM and keyboard-focusable at all times. It may be visually subdued until the header is hovered or contains keyboard focus, but discoverability must not depend on hover alone.
- Clicking the information action must stop event propagation so it does not also change sorting.
- Clicking the column label continues to cycle sorting and must not open the drawer.
- Opening the drawer must preserve the current rows, page, sorting, and scroll position.
- Use the existing right-side drawer surface. Column information and referenced-row information are mutually exclusive drawer modes; opening one replaces the other rather than stacking drawers.
- The drawer title is the exact case-preserved column name. Its context line is the schema-qualified relation name and relation kind, for example `public.orders · table`.
- Closing should work through the close button, backdrop, and `Escape`. Focus should return to the information button that opened it.

A dedicated information button is preferable to making the whole header open the drawer because the header already has sorting behavior. It also leaves room to evolve the trailing header action into a column-actions menu later if filtering, copying the column name, or other actions are added.

##### Required drawer information

The first release must show authoritative PostgreSQL metadata, not inferred labels.

**Overview**

- Exact, case-preserved column name.
- Ordinal position in the relation.
- Schema-qualified relation identity.
- Relation kind: table, partitioned table, view, materialized view, or foreign table.
- Exact PostgreSQL type rendered with `format_type`, including modifiers such as `character varying(100)`, `numeric(12,2)`, arrays, domains, enums, and schema-qualified custom types.
- Nullability.
- Non-default collation when one is explicitly assigned.
- Column comment, preserving multiline text.

**Value generation**

- Default expression using PostgreSQL's own deparsed expression.
- Identity status: `ALWAYS`, `BY DEFAULT`, or not an identity column.
- Generated status and generation expression. Support stored and virtual values when the connected PostgreSQL version exposes them.
- Do not present an absent default as the same thing as a generated or identity value.

**Constraints and relationships**

- Primary-key membership and position within a composite primary key.
- Unique-constraint membership and constraint name.
- Foreign-key membership, including the complete source column list, schema-qualified target relation, complete target column list, and `ON UPDATE`/`ON DELETE` actions.
- Check constraints associated with the column, shown by name and PostgreSQL definition.
- Composite constraints must be represented as one complete constraint. Do not flatten them into incorrect one-column relationships.

Use clear labels and compact sections rather than a raw catalog dump. Optional or unavailable fields should be omitted or shown as `Not applicable`; they must not be reported as false facts. Expressions, comments, names, and relationship targets must be naturally selectable and copyable.

##### Loading, caching, and errors

- Do not add this metadata to connection startup.
- Open the drawer immediately with a loading state, then fetch metadata lazily on the first column-information request for that relation.
- Prefer fetching and caching metadata for all columns in the relation in one relation-scoped request. This makes subsequent column clicks immediate and provides the data model for a future full Structure tab.
- Cache by exact schema and relation identity; relations with the same name in different schemas must never share metadata.
- A metadata failure affects only the drawer. Existing table data remains visible and usable.
- Show a concise error with a retry action. Do not close the drawer automatically.
- A later explicit metadata refresh must be able to invalidate this cache.

##### Suggested IPC model

Keep the DTO engine-agnostic even though PostgreSQL supplies the first implementation. A suitable shape is:

```text
RelationStructure
  schema
  relation
  relation_kind
  columns[]

ColumnInfo
  name
  ordinal_position
  data_type
  nullable
  collation?
  default_expression?
  identity?
  generated?
  generation_expression?
  comment?
  constraints[]

ColumnConstraint
  name
  kind: primary_key | unique | foreign_key | check
  definition
  column_position?
  source_columns[]
  target_schema?
  target_relation?
  target_columns[]
  on_update?
  on_delete?
```

The exact Rust representation may use enums and smaller nested types, but it must preserve the distinctions above and must not expose PostgreSQL driver types across the IPC boundary.

##### PostgreSQL metadata source

Use `pg_catalog`, with schema and relation supplied as query parameters. The implementation should be based on:

- `pg_class` and `pg_namespace` for exact relation identity and `relkind`.
- `pg_attribute` for column order, nullability, identity, generated state, type OID/modifier, and collation.
- `format_type(atttypid, atttypmod)` for exact type display.
- `pg_attrdef` with `pg_get_expr` for default and generation expressions.
- `col_description` for comments.
- `pg_constraint`, `unnest(... WITH ORDINALITY)`, and `pg_get_constraintdef` for key, foreign-key, and check information.

Exclude dropped and system columns. Preserve catalog order and exact identifier casing. Add no dependency; the existing PostgreSQL, Tauri, and Leptos paths are sufficient.

##### Acceptance criteria

1. Every visible data column has a keyboard-accessible information action in its header.
2. Activating it opens one drawer for the correct schema, relation, and column without changing sort or pagination.
3. Numeric modifiers, arrays, custom types, nullability, defaults, identity/generated properties, comments, and relation kind match PostgreSQL's catalogs.
4. Primary, unique, foreign-key, and check constraints are accurate; composite keys remain composite and ordered.
5. A column with little metadata still shows its name, position, type, and nullability without an empty or broken drawer.
6. Views and materialized views show valid column information without table-only assumptions.
7. Loading and errors are isolated from the Data view, and retry works.
8. A second column in the same relation opens from the relation-scoped cache without another metadata request.
9. Two same-named relations in different schemas never share cached metadata.
10. Drawer content is selectable and copyable, and the interaction is usable by keyboard.

##### Test expectations

- Add backend unit tests for catalog-code mappings such as relation kind, identity, generated state, and foreign-key actions.
- Add live PostgreSQL integration tests using the project database plus temporary fixture objects for quoted identifiers, type modifiers, arrays, enums/domains, comments, defaults, identity, generated columns, composite primary/unique/foreign keys, checks, views, and materialized views.
- Assert catalog results through the real `Connection` adapter; do not mock PostgreSQL behavior.
- Add frontend tests for drawer-state selection, relation-scoped cache keys, field presentation, and the separation between sort clicks and information-action clicks.

##### Explicitly out of scope for this slice

- Editing column definitions.
- Server-side filtering or a query-expression builder.
- Full index definitions and index usage statistics.
- Trigger details, partition trees, and complete view definitions.
- Storage statistics and per-column statistics.
- A full Structure tab. The relation-scoped response should enable that later, but this slice is the contextual column drawer only.

### 4. Missing everyday reader interactions

The main table surface should support:

- Server-side filtering.
- Server-side sorting across the complete result. **Resolved**
- Natural text selection and keyboard copying. **Resolved**
- Explicit refresh.
- Copy cell.
- Copy row.
- Full-value inspection for truncated values.
- Pretty JSON and array inspection.
- Configurable page size.
- Export of the current filtered result to CSV or JSON.

The application keeps global `user-select: none` behavior for desktop-like chrome and controls, while data cells and referenced-row values now explicitly restore text selection. Users can mark values naturally and copy them with the platform keyboard shortcut. Explicit copy-cell and copy-row actions remain open.

### 5. Weak database and schema context

The current sidebar and table tabs primarily display relation names without schema context. Databases commonly contain relations such as:

- `public.users`
- `auth.users`
- `audit.users`

These become visually ambiguous.

Videre should:

- Group objects by schema.
- Distinguish tables, views, and materialized views.
- Show schema-qualified names when needed.
- Preserve the exact casing of PostgreSQL identifiers.
- Highlight the active object in the sidebar.
- Add object search or quick-open.
- Display the active host, database, and user.
- Provide explicit disconnect and switch-connection actions.

### 6. Connection startup and metadata loading

After connection, `src-leptos/src/stores/db_store.rs` sequentially fetches tables, foreign keys and indexes for every relation, roles, and privileges before entering the connected shell.

This will scale poorly for databases containing many relations. A failure in secondary metadata also prevents the primary table browser from opening.

Recommended behavior:

- Open the shell after the connection and initial relation list succeed.
- Load relation metadata when a relation is opened.
- Load roles and global indexes only when requested.
- Let individual metadata failures degrade independently.
- Show loading progress and retry actions.
- Add explicit metadata refresh.

### 7. Index inspection is incomplete

The current index overview is useful as a summary but not as an authoritative index definition.

Missing information includes:

- Full `pg_get_indexdef` output.
- Expression terms.
- Partial-index predicates.
- Key columns versus included columns.
- Sort direction and null ordering.
- Collations and operator classes.
- Valid, ready, and live state.
- Constraint linkage.
- Usage statistics.

Expression-only indexes can be absent from the current query because it requires a matching table attribute.

### 8. Role summaries can be misleading

The current Roles page summarizes direct table grants. It does not calculate effective access through membership, ownership, `PUBLIC`, database privileges, or schema privileges.

The UI should either compute effective access or clearly label the information as **direct table grants**. Labels such as “no table access” should not imply an authoritative permission result when inherited access may exist.

Roles are useful, but the current feature is disproportionately polished compared with the central table-reading workflow.

### 9. Relationship navigation should go further

The current FK drawer is a strong start. A complete relationship workflow should add:

- Schema and relation context in the drawer.
- The source and target columns used for the relationship.
- Open referenced relation in a tab.
- Continue following relationships from the referenced row.
- Navigation history or breadcrumbs.
- Reverse relationships: which relations and rows reference this record.
- Correct support for composite foreign keys.

### 10. Some product areas are overbuilt relative to the core

Themes, role details, custom window chrome, and editor-like empty tabs have received considerable attention while refresh, filtering, copying, and relation structure are still absent.

The existing themes are worth keeping, but additional visual expansion should pause until the core reading experience is excellent.

Empty “Untitled” tabs do not currently have a meaningful task. They make sense in a query editor but not in a read-only object browser. Object tabs should normally be created by opening an actual database object.

## Read-only observability and diagnostics

Operational information can still belong in Reader Mode because reading PostgreSQL statistics does not require edit controls.

A focused live-diagnostics layer could contain three pages.

### Activity

- Session PID.
- Database and user.
- Application name.
- Session state.
- Current query.
- Query and transaction duration.
- Wait event type and wait event.
- Idle-in-transaction visibility.

### Locks

- Blocked session.
- Blocking session.
- Lock or wait duration.
- Related queries.
- A simple blocker chain.

### Health

Per database and relation:

- Heap, index, TOAST, and total size.
- Estimated live and dead tuples.
- Sequential and index scan counts.
- Last vacuum and autovacuum.
- Last analyze and autoanalyze.
- Vacuum and analyze counts.
- Active maintenance progress where supported.

Activity, Locks, and Health would create a coherent “what is happening now?” capability without turning Videre into a historical monitoring platform.

Later capabilities can include:

- `pg_stat_statements` query insights.
- Structured `EXPLAIN` rendering.
- Replication and WAL status.
- Historical sampling and trends.

## Recommended information architecture

```text
Current connection: localhost / app_production
Quick open…

Schemas
  public
    Tables
    Views
  billing
    Tables
    Views

Database
  Activity
  Locks
  Health
  Indexes
  Roles

Settings
```

Inside a relation tab:

```text
public.orders

Data | Structure | Relationships | Indexes
```

The global Indexes page can remain as an overview, but each index should link back to its owning relation.

## Prioritized roadmap

### P0 — Trustworthy Reader Mode

1. [x] Deterministic PostgreSQL ordering and pagination.
2. [x] Server-side sorting across the complete result.
3. [ ] Server-side filtering. **Deferred pending query-model design**
4. [ ] Refresh and reliable loading/error states.
5. [ ] Copy cell, copy row, and full-value inspection.
6. [ ] Schema-qualified relation identity.
7. [ ] Active connection identity and lifecycle controls.
8. [ ] Lazy metadata loading.

### P1 — Understand database objects

1. [ ] Relation structure and types. **Next read-only slice**
2. Complete constraints.
3. Full index definitions.
4. Enhanced forward and reverse relationship navigation.
5. Views and materialized-view definitions.
6. Triggers and partitions.
7. Relation sizes and row estimates.
8. Quick-open and scalable schema navigation.

### P2 — Live diagnostics within Reader Mode

1. Activity.
2. Locks and blockers.
3. Storage and maintenance health.
4. Index usage.
5. Optional `pg_stat_statements` support.
6. Basic structured `EXPLAIN`.

### P3 — Edit Mode

1. Explicit mode selection.
2. Insert rows.
3. Update rows.
4. Delete rows.
5. Change previews and clear confirmations.
6. A limited query workspace if it supports the focused product workflow.

### P4 — Broader platform support

1. Validate the product with real PostgreSQL users.
2. Stabilize a database-agnostic capability model.
3. Evaluate SQLite and MySQL based on user demand.

## Features to defer

To preserve focus, do not prioritize:

- More visual themes.
- A full SQL IDE.
- Migration management.
- Schema editing and diagram design.
- Dashboard builders.
- Alerts and historical monitoring infrastructure.
- AI-generated database recommendations.
- Additional database engines before PostgreSQL is excellent.

## Product success criteria

Reader Mode is ready when a developer can:

1. Connect to a small or large PostgreSQL database without a long blocking startup.
2. Find an object quickly regardless of schema size.
3. Trust pagination, sorting, and filtering.
4. Inspect and copy any value without fighting the interface.
5. Understand a relation without opening another database tool.
6. Follow relationships across several records without writing SQL.
7. Identify a long-running or blocked query from a clear live view.
8. Always know which connection and mode are active.

Edit Mode should begin only after this experience is reliable and polished.

## Final assessment

Videre is not missing a good foundation. It is missing depth in the central workflow.

The strongest path is:

> Connect, find any PostgreSQL object quickly, inspect its data accurately, understand its structure, and follow its relationships—all in a calm, beautiful, read-only interface.

After that experience is exceptional, Edit Mode can extend Videre without displacing Reader Mode as the product's default and defining experience.

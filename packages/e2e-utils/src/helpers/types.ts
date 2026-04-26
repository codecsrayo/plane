/**
 * DTOs tipados extraídos de los structs Rust del backend.
 * Fuente: `apps/api_rust/src/routes/issues.rs`
 *
 * Usados en tests para `toMatchObject()` con shapes exactos.
 */

// ── Issue ─────────────────────────────────────────────────────────────────────

/**
 * Campos requeridos para crear un issue.
 * Fuente: `CreateIssueRequest` — solo `name` es obligatorio.
 */
export interface IssueCreatePayload {
  name: string;
  description_html?: string;
  priority?: "none" | "urgent" | "high" | "medium" | "low";
  state_id?: string;
  parent_id?: string;
  start_date?: string;       // YYYY-MM-DD
  target_date?: string;      // YYYY-MM-DD
  estimate_point?: string;   // UUID del EstimatePoint
  type_id?: string;
  assignee_ids?: string[];
  label_ids?: string[];
}

/**
 * Shape mínimo garantizado de la respuesta POST /issues (IssueCreateResponse).
 * El backend siempre devuelve estos campos; el resto pueden ser null.
 */
export interface IssueCreateShape {
  id: string;
  name: string;
  priority: string;          // "none" por defecto
  sequence_id: number;
  project_id: string;
  is_draft: boolean;
  attachment_count: number;
  link_count: number;
  sub_issues_count: number;
  module_ids: string[];
  label_ids: string[];
  assignee_ids: string[];
}

/**
 * Shape extendido de GET /issues/{pk} (IssueDetailResponse).
 * Añade description_html, is_subscribed, is_intake que CREATE no incluye.
 */
export interface IssueDetailShape extends IssueCreateShape {
  description_html: string;
  is_subscribed: boolean;
  is_intake: boolean;
  cycle_id: string | null;
}

// ── Paginación ────────────────────────────────────────────────────────────────

/**
 * Formato del cursor de paginación.
 * Fuente: `src/routes/issue_pagination.rs`
 * Formato: `{page_size}:{page}:{is_prev}` — e.g. `"100:0:0"`
 *
 * DEFAULT_PER_PAGE = 100
 */
export interface PaginatedResponse<T> {
  results: T[];
  next_cursor: string;   // "100:1:0"
  prev_cursor: string;   // "100:-1:1" en primera página
  next_page_results: boolean;
  prev_page_results: boolean;
  total_results: number;
  extra_stats?: Record<string, unknown>;
  grouped_by?: string;
  sub_grouped_by?: string;
}

/** Parsea un cursor al formato `page_size:page:is_prev` */
export function parseCursor(cursor: string): { pageSize: number; page: number; isPrev: boolean } {
  const [ps, pg, ip] = cursor.split(":").map(Number);
  return { pageSize: ps, page: pg, isPrev: ip === 1 };
}

/** Construye un cursor para pasar como query param `?cursor=` */
export function buildCursor(pageSize = 100, page = 0, isPrev = false): string {
  return `${pageSize}:${page}:${isPrev ? 1 : 0}`;
}

// ── Workspace ─────────────────────────────────────────────────────────────────

export interface WorkspaceShape {
  id: string;
  name: string;
  slug: string;
}

// ── Member ────────────────────────────────────────────────────────────────────

export interface WorkspaceMemberShape {
  id: string;
  role: 5 | 10 | 15 | 20;  // GUEST | VIEWER | MEMBER | ADMIN
  member: string;            // user UUID
}

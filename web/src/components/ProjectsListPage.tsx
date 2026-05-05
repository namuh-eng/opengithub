import Link from "next/link";
import type { ProjectList, ProjectRow, ProjectTemplateRow } from "@/lib/api";

type ProjectsListPageProps = {
  list: ProjectList;
  scopeLabel?: string;
};

function formatDate(value: string) {
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(new Date(value));
}

function stateLabel(state: string) {
  return state === "closed" ? "Closed" : "Open";
}

function statusChip(status: ProjectRow["status"]) {
  if (!status) {
    return null;
  }
  const chipClass =
    status.status === "on_track" || status.status === "complete"
      ? "chip ok"
      : status.status === "at_risk"
        ? "chip warn"
        : status.status === "off_track"
          ? "chip err"
          : "chip soft";
  return <span className={chipClass}>{status.label}</span>;
}

const PROJECT_SORT_OPTIONS = [
  { value: "recently_updated", label: "Recently updated" },
  { value: "name_asc", label: "Name A-Z" },
  { value: "name_desc", label: "Name Z-A" },
  { value: "created_desc", label: "Newest" },
  { value: "created_asc", label: "Oldest" },
];

function filterHref(
  list: ProjectList,
  overrides: Partial<{
    q: string | null;
    state: string | null;
    tab: string | null;
    sort: string | null;
    page: number | null;
  }> = {},
) {
  const [basePath, baseQuery = ""] = list.scope.href.split("?");
  const params = new URLSearchParams(baseQuery);
  const tabParam = list.scope.kind === "user" ? "projectTab" : "tab";
  const next = {
    q: list.filters.query,
    state: list.filters.state,
    tab: list.filters.tab,
    sort: list.filters.sort,
    page: list.filters.page,
    ...overrides,
  };

  if (next.q?.trim()) {
    params.set("q", next.q.trim());
  }
  if (next.state && next.state !== "open") {
    params.set("state", next.state);
  } else {
    params.delete("state");
  }
  if (next.tab && next.tab !== "projects") {
    params.set(tabParam, next.tab);
  } else {
    params.delete(tabParam);
  }
  if (next.sort && next.sort !== "recently_updated") {
    params.set("sort", next.sort);
  } else {
    params.delete("sort");
  }
  if (next.page && next.page > 1) {
    params.set("page", String(next.page));
  } else {
    params.delete("page");
  }

  const suffix = params.size ? `?${params.toString()}` : "";
  return `${basePath}${suffix}`;
}

function projectMeta(project: ProjectRow) {
  const parts = [
    `#${project.number}`,
    `Updated ${formatDate(project.updatedAt)}`,
    project.defaultRepository
      ? `Default ${project.defaultRepository.fullName}`
      : null,
    project.linkedRepositoriesCount > 0
      ? `${project.linkedRepositoriesCount.toLocaleString()} linked repositories`
      : null,
    `${project.counts.total.toLocaleString()} items`,
  ].filter(Boolean);
  return parts.join(" · ");
}

function ProjectsTabs({ list }: { list: ProjectList }) {
  const projectActive = list.filters.tab !== "templates";
  return (
    <nav className="tabs" aria-label="Project list sections">
      <Link
        aria-current={projectActive ? "page" : undefined}
        className={`tab ${projectActive ? "active" : ""}`}
        href={filterHref(list, { tab: "projects", page: 1 })}
      >
        Projects{" "}
        <span className="t-num">{list.counts.total.toLocaleString()}</span>
      </Link>
      <Link
        aria-current={!projectActive ? "page" : undefined}
        className={`tab ${!projectActive ? "active" : ""}`}
        href={filterHref(list, { tab: "templates", page: 1 })}
      >
        Templates{" "}
        <span className="t-num">{list.counts.templates.toLocaleString()}</span>
      </Link>
    </nav>
  );
}

function StateTabs({ list }: { list: ProjectList }) {
  const state = list.filters.state === "closed" ? "closed" : "open";
  return (
    <nav className="flex flex-wrap gap-2" aria-label="Project state filters">
      <Link
        aria-current={state === "open" ? "page" : undefined}
        className={`chip ${state === "open" ? "active" : "soft"}`}
        href={filterHref(list, { state: "open", page: 1 })}
      >
        Open <span className="t-num">{list.counts.open}</span>
      </Link>
      <Link
        aria-current={state === "closed" ? "page" : undefined}
        className={`chip ${state === "closed" ? "active" : "soft"}`}
        href={filterHref(list, { state: "closed", page: 1 })}
      >
        Closed <span className="t-num">{list.counts.closed}</span>
      </Link>
    </nav>
  );
}

function ActiveFilters({ list }: { list: ProjectList }) {
  const filters = [
    list.filters.query
      ? {
          label: `Search: ${list.filters.query}`,
          href: filterHref(list, { q: null, page: 1 }),
        }
      : null,
    list.filters.state === "closed"
      ? {
          label: "Closed projects",
          href: filterHref(list, { state: "open", page: 1 }),
        }
      : null,
    list.filters.tab === "templates"
      ? {
          label: "Templates",
          href: filterHref(list, { tab: "projects", page: 1 }),
        }
      : null,
    list.filters.sort !== "recently_updated"
      ? {
          label:
            PROJECT_SORT_OPTIONS.find(
              (option) => option.value === list.filters.sort,
            )?.label ?? list.filters.sort,
          href: filterHref(list, { sort: "recently_updated", page: 1 }),
        }
      : null,
  ].filter(Boolean) as Array<{ label: string; href: string }>;

  if (filters.length === 0) {
    return null;
  }

  return (
    <fieldset className="flex flex-wrap items-center gap-2">
      <legend className="sr-only">Active project filters</legend>
      {filters.map((filter) => (
        <Link
          className="chip soft no-underline"
          href={filter.href}
          key={filter.label}
        >
          {filter.label} x
        </Link>
      ))}
      <Link
        className="btn sm ghost"
        href={filterHref(list, {
          q: null,
          state: "open",
          tab: "projects",
          sort: "recently_updated",
          page: 1,
        })}
      >
        Clear filters
      </Link>
    </fieldset>
  );
}

function Pagination({ list }: { list: ProjectList }) {
  const showingTemplates = list.filters.tab === "templates";
  const total = showingTemplates ? list.templates.total : list.total;
  const pageSize = showingTemplates ? list.templates.pageSize : list.pageSize;
  const page = showingTemplates ? list.templates.page : list.page;
  const totalPages = Math.max(1, Math.ceil(total / Math.max(1, pageSize)));

  if (totalPages <= 1) {
    return null;
  }

  return (
    <nav
      className="flex flex-wrap items-center justify-between gap-3 py-4"
      aria-label="Projects pagination"
    >
      <p className="t-sm" style={{ color: "var(--ink-3)" }}>
        Page <span className="t-num">{page}</span> of{" "}
        <span className="t-num">{totalPages}</span>
      </p>
      <div className="flex gap-2">
        <Link
          aria-disabled={page <= 1}
          className={`btn sm ${page <= 1 ? "disabled" : "ghost"}`}
          href={
            page <= 1 ? filterHref(list) : filterHref(list, { page: page - 1 })
          }
        >
          Previous
        </Link>
        <Link
          aria-disabled={page >= totalPages}
          className={`btn sm ${page >= totalPages ? "disabled" : "ghost"}`}
          href={
            page >= totalPages
              ? filterHref(list)
              : filterHref(list, { page: page + 1 })
          }
        >
          Next
        </Link>
      </div>
    </nav>
  );
}

function ProjectRowView({ project }: { project: ProjectRow }) {
  return (
    <article className="list-row grid gap-3 py-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-start">
      <Link className="min-w-0 no-underline" href={project.workspaceHref}>
        <div className="flex flex-wrap items-center gap-2">
          <h3 className="t-h3 truncate">{project.title}</h3>
          <span
            className={project.state === "closed" ? "chip soft" : "chip ok"}
          >
            {stateLabel(project.state)}
          </span>
          {project.visibility !== "public" ? (
            <span className="chip soft">{project.visibility}</span>
          ) : null}
          {project.isTemplate ? (
            <span className="chip accent">Template</span>
          ) : null}
          {statusChip(project.status)}
        </div>
        {project.description ? (
          <p
            className="t-sm mt-2 line-clamp-2"
            style={{ color: "var(--ink-2)" }}
          >
            {project.description}
          </p>
        ) : null}
        <p className="t-mono-sm mt-2" style={{ color: "var(--ink-3)" }}>
          {projectMeta(project)}
        </p>
      </Link>
      <div className="flex flex-wrap gap-2 md:justify-end">
        <Link className="btn sm ghost" href={project.workspaceHref}>
          Open
        </Link>
        <button
          className="btn sm"
          disabled={!project.viewerCanCopy}
          title={
            project.viewerCanCopy
              ? "Copy flow arrives in the next Projects phase"
              : "You need write access to copy this project"
          }
          type="button"
        >
          Copy
        </button>
      </div>
    </article>
  );
}

function TemplateRowView({ template }: { template: ProjectTemplateRow }) {
  return (
    <article className="list-row grid gap-3 py-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-start">
      <Link className="min-w-0 no-underline" href={template.projectHref}>
        <div className="flex flex-wrap items-center gap-2">
          <h3 className="t-h3 truncate">{template.title}</h3>
          <span className={template.isPublic ? "chip ok" : "chip soft"}>
            {template.isPublic ? "Public template" : "Private template"}
          </span>
        </div>
        {template.description ? (
          <p
            className="t-sm mt-2 line-clamp-2"
            style={{ color: "var(--ink-2)" }}
          >
            {template.description}
          </p>
        ) : null}
        <p className="t-mono-sm mt-2" style={{ color: "var(--ink-3)" }}>
          Source {template.projectTitle} · Created{" "}
          {formatDate(template.createdAt)}
        </p>
      </Link>
      <button
        className="btn sm"
        disabled={!template.viewerCanCopy}
        title={
          template.viewerCanCopy
            ? "Copy flow arrives in the next Projects phase"
            : "You need write access to copy this template"
        }
        type="button"
      >
        Copy
      </button>
    </article>
  );
}

export function ProjectsListPage({
  list,
  scopeLabel = list.scope.login,
}: ProjectsListPageProps) {
  const showingTemplates = list.filters.tab === "templates";
  const rows = showingTemplates ? list.templates.items : list.items;
  const unavailable = list.unavailableReason;
  const [formAction, formBaseQuery = ""] = list.scope.href.split("?");
  const formBaseParams = new URLSearchParams(formBaseQuery);
  const tabParam = list.scope.kind === "user" ? "projectTab" : "tab";

  return (
    <section className="grid gap-5" aria-labelledby="projects-list-title">
      <div className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-end">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Projects
          </p>
          <h2 className="t-h2 mt-1" id="projects-list-title">
            {scopeLabel}
          </h2>
          <p className="t-body mt-2" style={{ color: "var(--ink-2)" }}>
            Track tables, boards, and roadmap plans connected to this scope.
          </p>
        </div>
        {list.viewerPermissions.canCreate ? (
          <Link className="btn primary" href={`${list.scope.href}/new`}>
            New project
          </Link>
        ) : (
          <button className="btn" disabled type="button">
            New project
          </button>
        )}
      </div>

      <div className="card p-4">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Welcome to Projects
            </p>
            <p className="t-sm mt-2" style={{ color: "var(--ink-2)" }}>
              Build planning views from issues, pull requests, and draft work
              while keeping repository links visible.
            </p>
          </div>
          <span className="chip soft">
            {list.viewerPermissions.viewerRole ?? "viewer"}
          </span>
        </div>
      </div>

      {unavailable ? (
        <div className="card p-5" role="status">
          <span className="chip warn">Unavailable</span>
          <p className="t-body mt-3" style={{ color: "var(--ink-2)" }}>
            {unavailable}
          </p>
        </div>
      ) : null}

      <div className="card overflow-hidden">
        <div
          className="border-b px-5 pt-1"
          style={{ borderColor: "var(--line)" }}
        >
          <ProjectsTabs list={list} />
        </div>
        <div
          className="grid gap-3 border-b p-4"
          style={{ borderColor: "var(--line-soft)" }}
        >
          <form
            action={formAction}
            className="grid gap-3 md:grid-cols-[minmax(0,1fr)_180px_auto]"
          >
            {Array.from(formBaseParams.entries()).map(([name, value]) => (
              <input
                key={`${name}:${value}`}
                name={name}
                type="hidden"
                value={value}
              />
            ))}
            {list.filters.tab !== "projects" ? (
              <input name={tabParam} type="hidden" value={list.filters.tab} />
            ) : null}
            {list.filters.state !== "open" ? (
              <input name="state" type="hidden" value={list.filters.state} />
            ) : null}
            <div className="input">
              <input
                aria-label="Search all projects"
                defaultValue={list.filters.query ?? ""}
                name="q"
                placeholder="Search all projects"
              />
            </div>
            <label className="input">
              <span className="sr-only">Sort projects</span>
              <select
                aria-label="Sort projects"
                defaultValue={list.filters.sort}
                name="sort"
              >
                {PROJECT_SORT_OPTIONS.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </label>
            <button className="btn" type="submit">
              Apply
            </button>
          </form>
          <div className="flex flex-wrap items-center justify-between gap-3">
            <StateTabs list={list} />
            <ActiveFilters list={list} />
          </div>
        </div>
        <div className="px-5">
          {rows.length > 0 ? (
            rows.map((row) =>
              showingTemplates ? (
                <TemplateRowView
                  key={(row as ProjectTemplateRow).id}
                  template={row as ProjectTemplateRow}
                />
              ) : (
                <ProjectRowView
                  key={(row as ProjectRow).id}
                  project={row as ProjectRow}
                />
              ),
            )
          ) : (
            <div className="py-10 text-center">
              <p className="t-h3">
                No {showingTemplates ? "templates" : "projects"} match this
                view.
              </p>
              <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
                Adjust filters or create a project when you have permission.
              </p>
            </div>
          )}
          <Pagination list={list} />
        </div>
      </div>
    </section>
  );
}

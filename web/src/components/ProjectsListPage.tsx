"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { type FormEvent, useState } from "react";
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

function projectInsightsHref(project: ProjectRow) {
  return project.workspaceHref.replace(/\/views\/\d+.*$/, "/insights");
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

type CopyTarget = {
  id: string;
  title: string;
  kind: "project" | "template";
  viewerCanCopy: boolean;
};

function ProjectRowView({
  project,
  onCopy,
}: {
  project: ProjectRow;
  onCopy: (target: CopyTarget) => void;
}) {
  const [menuOpen, setMenuOpen] = useState(false);

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
        <Link className="btn sm ghost" href={projectInsightsHref(project)}>
          Insights
        </Link>
        <Link className="btn sm ghost" href={project.workspaceHref}>
          Open
        </Link>
        <div className="relative">
          <button
            aria-expanded={menuOpen}
            className="btn sm"
            onClick={() => setMenuOpen((open) => !open)}
            type="button"
          >
            More project options
          </button>
          {menuOpen ? (
            <div
              className="card absolute right-0 z-10 mt-2 grid min-w-48 gap-2 p-2"
              style={{ background: "var(--surface)" }}
            >
              <button
                className="btn sm ghost justify-start"
                disabled={!project.viewerCanCopy}
                onClick={() =>
                  onCopy({
                    id: project.id,
                    title: project.title,
                    kind: "project",
                    viewerCanCopy: project.viewerCanCopy,
                  })
                }
                title="Copy this project"
                type="button"
              >
                Copy
              </button>
            </div>
          ) : null}
        </div>
      </div>
    </article>
  );
}

function TemplateRowView({
  template,
  onCopy,
}: {
  template: ProjectTemplateRow;
  onCopy: (target: CopyTarget) => void;
}) {
  const [menuOpen, setMenuOpen] = useState(false);

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
      <div className="relative justify-self-start md:justify-self-end">
        <button
          aria-expanded={menuOpen}
          className="btn sm"
          onClick={() => setMenuOpen((open) => !open)}
          type="button"
        >
          More project options
        </button>
        {menuOpen ? (
          <div
            className="card absolute right-0 z-10 mt-2 grid min-w-48 gap-2 p-2"
            style={{ background: "var(--surface)" }}
          >
            <button
              className="btn sm ghost justify-start"
              disabled={!template.viewerCanCopy}
              onClick={() =>
                onCopy({
                  id: template.projectId,
                  title: template.title,
                  kind: "template",
                  viewerCanCopy: template.viewerCanCopy,
                })
              }
              title="Copy this template"
              type="button"
            >
              Copy
            </button>
          </div>
        ) : null}
      </div>
    </article>
  );
}

function CopyProjectDialog({
  target,
  onClose,
}: {
  target: CopyTarget;
  onClose: () => void;
}) {
  const router = useRouter();
  const [title, setTitle] = useState(`[COPY] ${target.title}`);
  const [includeDraftIssues, setIncludeDraftIssues] = useState(true);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const normalizedTitle = title.trim();
    if (!normalizedTitle) {
      setError("Project title is required.");
      return;
    }
    setPending(true);
    setError(null);
    const response = await fetch(
      `/api/projects/${encodeURIComponent(target.id)}/copies`,
      {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          title: normalizedTitle,
          includeDraftIssues,
        }),
      },
    );
    const body = await response.json().catch(() => null);
    if (!response.ok) {
      setPending(false);
      setError(
        body?.error?.message ??
          "Project could not be copied. Check your permissions and try again.",
      );
      return;
    }
    router.push(body.workspaceHref ?? body.href);
  }

  return (
    <div
      aria-labelledby="copy-project-title"
      aria-modal="true"
      className="fixed inset-0 z-50 grid place-items-center px-4"
      role="dialog"
      style={{
        background: "color-mix(in oklch, var(--ink-1) 28%, transparent)",
      }}
    >
      <form className="card grid w-full max-w-lg gap-4 p-5" onSubmit={submit}>
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Copy {target.kind}
          </p>
          <h3 className="t-h2 mt-1" id="copy-project-title">
            {target.title}
          </h3>
        </div>
        <label className="grid gap-2">
          <span className="t-sm">Project title</span>
          <span className="input">
            <input
              onChange={(event) => setTitle(event.target.value)}
              value={title}
            />
          </span>
        </label>
        <label className="flex items-start gap-3 t-sm">
          <input
            checked={includeDraftIssues}
            className="mt-1"
            onChange={(event) => setIncludeDraftIssues(event.target.checked)}
            type="checkbox"
          />
          <span>
            Include draft issues
            <span className="block t-xs">
              Linked issues and pull requests stay in the source project.
            </span>
          </span>
        </label>
        {error ? (
          <p className="chip err justify-self-start" role="alert">
            {error}
          </p>
        ) : null}
        <div className="flex flex-wrap justify-end gap-2">
          <button
            className="btn"
            disabled={pending}
            onClick={onClose}
            type="button"
          >
            Cancel
          </button>
          <button className="btn primary" disabled={pending} type="submit">
            {pending ? "Copying..." : "Copy project"}
          </button>
        </div>
      </form>
    </div>
  );
}

export function ProjectsListPage({
  list,
  scopeLabel = list.scope.login,
}: ProjectsListPageProps) {
  const [copyTarget, setCopyTarget] = useState<CopyTarget | null>(null);
  const [welcomeDismissed, setWelcomeDismissed] = useState(false);
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

      {!welcomeDismissed ? (
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
            <div className="flex flex-wrap items-center gap-2">
              <span className="chip soft">
                {list.viewerPermissions.viewerRole ?? "viewer"}
              </span>
              <button
                aria-label="Dismiss Welcome to Projects"
                className="btn sm ghost"
                onClick={() => setWelcomeDismissed(true)}
                type="button"
              >
                Dismiss
              </button>
            </div>
          </div>
        </div>
      ) : null}

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
                  onCopy={setCopyTarget}
                  template={row as ProjectTemplateRow}
                />
              ) : (
                <ProjectRowView
                  key={(row as ProjectRow).id}
                  onCopy={setCopyTarget}
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
      {copyTarget ? (
        <CopyProjectDialog
          key={copyTarget.id}
          onClose={() => setCopyTarget(null)}
          target={copyTarget}
        />
      ) : null}
    </section>
  );
}

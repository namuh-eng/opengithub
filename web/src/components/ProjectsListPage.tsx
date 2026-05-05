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
        href={list.scope.href}
      >
        Projects{" "}
        <span className="t-num">{list.counts.total.toLocaleString()}</span>
      </Link>
      <Link
        aria-current={!projectActive ? "page" : undefined}
        className={`tab ${!projectActive ? "active" : ""}`}
        href={`${list.scope.href}?tab=templates`}
      >
        Templates{" "}
        <span className="t-num">{list.counts.templates.toLocaleString()}</span>
      </Link>
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
          className="grid gap-3 border-b p-4 md:grid-cols-[minmax(0,1fr)_auto_auto]"
          style={{ borderColor: "var(--line-soft)" }}
        >
          <div className="input">
            <input
              aria-label="Search all projects"
              defaultValue={list.filters.query ?? ""}
              name="q"
              placeholder="Search all projects"
              readOnly
            />
          </div>
          <span className="chip soft">
            Open <span className="t-num">{list.counts.open}</span>
          </span>
          <span className="chip soft">
            Closed <span className="t-num">{list.counts.closed}</span>
          </span>
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
        </div>
      </div>
    </section>
  );
}

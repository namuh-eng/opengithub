import Link from "next/link";
import { MarkdownBody } from "@/components/MarkdownBody";
import { RepositoryReleaseInteractions } from "@/components/RepositoryReleaseInteractions";
import { RepositoryShell } from "@/components/RepositoryShell";
import type {
  ApiErrorEnvelope,
  ListEnvelope,
  ReleaseTagSummary,
  RepositoryOverview,
  RepositoryReleaseDetail,
  RepositoryReleaseSummary,
} from "@/lib/api";
import { repositoryCompareRangeHref } from "@/lib/navigation";

type RepositoryReleasesPageProps = {
  authenticated: boolean;
  repository: RepositoryOverview;
  releases: ListEnvelope<RepositoryReleaseSummary> | ApiErrorEnvelope;
};

type RepositoryReleaseDetailPageProps = {
  authenticated: boolean;
  repository: RepositoryOverview;
  release: RepositoryReleaseDetail | ApiErrorEnvelope;
};

type RepositoryTagsPageProps = {
  repository: RepositoryOverview;
  tags: ListEnvelope<ReleaseTagSummary> | ApiErrorEnvelope;
};

function isApiError(value: unknown): value is ApiErrorEnvelope {
  return Boolean(value && typeof value === "object" && "error" in value);
}

function basePath(repository: RepositoryOverview) {
  return `/${repository.owner_login}/${repository.name}`;
}

function formatDate(value: string | null) {
  if (!value) return "Unpublished";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "recently";
  return new Intl.DateTimeFormat("en", {
    day: "numeric",
    month: "short",
    year: "numeric",
  }).format(date);
}

function initials(login: string) {
  return login
    .split(/[-_.\s@]+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase())
    .join("");
}

function compareHref(repository: RepositoryOverview, tagName: string) {
  return repositoryCompareRangeHref(
    repository.owner_login,
    repository.name,
    tagName,
    repository.default_branch,
  );
}

function ReleasesTabs({
  active,
  repository,
}: {
  active: "releases" | "tags";
  repository: RepositoryOverview;
}) {
  const base = basePath(repository);
  return (
    <nav aria-label="Releases and tags" className="tabs mb-6">
      <Link
        aria-current={active === "releases" ? "page" : undefined}
        className={`tab ${active === "releases" ? "active" : ""}`}
        href={`${base}/releases`}
      >
        Releases
      </Link>
      <Link
        aria-current={active === "tags" ? "page" : undefined}
        className={`tab ${active === "tags" ? "active" : ""}`}
        href={`${base}/tags`}
      >
        Tags
      </Link>
    </nav>
  );
}

function ReleaseUnavailable({
  error,
  title,
}: {
  error: ApiErrorEnvelope;
  title: string;
}) {
  const forbidden = error.status === 403;
  return (
    <section className="card p-6" role="status">
      <span className={`chip ${forbidden ? "warn" : "err"}`}>
        {forbidden ? "Access restricted" : "Unavailable"}
      </span>
      <h2 className="t-h2 mt-4">{title}</h2>
      <p className="t-body mt-3" style={{ color: "var(--ink-2)" }}>
        {forbidden
          ? "Private repository release metadata is only visible to viewers with read access."
          : error.error.message}
      </p>
    </section>
  );
}

function ReleaseBadges({ release }: { release: RepositoryReleaseSummary }) {
  return (
    <div className="flex flex-wrap gap-2">
      {release.latest ? <span className="chip accent">Latest</span> : null}
      {release.prerelease ? (
        <span className="chip warn">Pre-release</span>
      ) : null}
      {release.draft ? <span className="chip soft">Draft</span> : null}
      {release.verified ? <span className="chip ok">Verified</span> : null}
    </div>
  );
}

function AuthorLine({ release }: { release: RepositoryReleaseSummary }) {
  return (
    <div
      className="mt-3 flex flex-wrap items-center gap-2 t-sm"
      style={{ color: "var(--ink-3)" }}
    >
      <span className="av sm">{initials(release.author.login) || "OG"}</span>
      <span>{release.author.displayName ?? release.author.login}</span>
      <span>published {formatDate(release.publishedAt)}</span>
      {release.shortOid ? (
        <Link
          className="t-mono-sm hover:underline"
          href={`${basePathFromRelease(release)}/commit/${release.targetOid}`}
        >
          {release.shortOid}
        </Link>
      ) : null}
    </div>
  );
}

function basePathFromRelease(release: RepositoryReleaseSummary) {
  const parts = release.links.htmlHref.split("/").filter(Boolean);
  return parts.length >= 2 ? `/${parts[0]}/${parts[1]}` : "";
}

function ContributorRow({
  contributors,
}: {
  contributors: RepositoryReleaseSummary["contributors"];
}) {
  if (contributors.length === 0) return null;
  return (
    <div className="mt-4 flex items-center gap-2">
      <span className="t-xs">Contributors</span>
      <div className="flex -space-x-1">
        {contributors.map((contributor) => (
          <span
            className="av sm border"
            key={contributor.id}
            style={{ borderColor: "var(--surface)" }}
            title={contributor.displayName ?? contributor.login}
          >
            {initials(contributor.login) || "OG"}
          </span>
        ))}
      </div>
    </div>
  );
}

function ReleaseCard({
  authenticated,
  detailHtml,
  release,
  repository,
}: {
  authenticated: boolean;
  detailHtml?: string;
  release: RepositoryReleaseSummary;
  repository: RepositoryOverview;
}) {
  return (
    <article className="card p-5">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div className="min-w-0">
          <Link
            className="t-mono-sm hover:underline"
            href={release.links.tagHref}
            style={{ color: "var(--accent)" }}
          >
            {release.tagName}
          </Link>
          <h2 className="t-h2 mt-1">
            <Link className="hover:underline" href={release.links.htmlHref}>
              {release.title}
            </Link>
          </h2>
        </div>
        <ReleaseBadges release={release} />
      </div>
      <AuthorLine release={release} />
      <div className="mt-5">
        {detailHtml ? (
          <MarkdownBody html={detailHtml} />
        ) : release.bodyExcerpt ? (
          <MarkdownBody html={release.bodyExcerpt} />
        ) : (
          <p className="t-sm" style={{ color: "var(--ink-3)" }}>
            No release notes were provided.
          </p>
        )}
      </div>
      <ContributorRow contributors={release.contributors} />
      <RepositoryReleaseInteractions
        authenticated={authenticated}
        release={release}
        repository={repository}
      />
    </article>
  );
}

function Pagination({
  envelope,
  hrefBase,
}: {
  envelope: ListEnvelope<unknown>;
  hrefBase: string;
}) {
  const previous = envelope.page > 1;
  const next = envelope.page * envelope.pageSize < envelope.total;
  if (!previous && !next) return null;
  return (
    <nav aria-label="Pagination" className="mt-6 flex justify-between">
      {previous ? (
        <Link className="btn" href={`${hrefBase}?page=${envelope.page - 1}`}>
          Previous
        </Link>
      ) : (
        <span />
      )}
      {next ? (
        <Link className="btn" href={`${hrefBase}?page=${envelope.page + 1}`}>
          Next
        </Link>
      ) : null}
    </nav>
  );
}

export function RepositoryReleasesPage({
  authenticated,
  releases,
  repository,
}: RepositoryReleasesPageProps) {
  return (
    <RepositoryShell
      activePath={`${basePath(repository)}/releases`}
      frameClassName="max-w-5xl"
      repository={repository}
    >
      <section>
        <ReleasesTabs active="releases" repository={repository} />
        <div className="mb-6 flex flex-wrap items-end justify-between gap-4">
          <div>
            <p className="t-label">Repository releases</p>
            <h1 className="t-h1 mt-2">Releases</h1>
          </div>
          <Link
            className="btn"
            href={`${basePath(repository)}/releases/latest`}
          >
            Latest release
          </Link>
        </div>
        {isApiError(releases) ? (
          <ReleaseUnavailable
            error={releases}
            title="Releases could not load"
          />
        ) : releases.items.length === 0 ? (
          <div className="card p-6" role="status">
            <span className="chip soft">No releases</span>
            <h2 className="t-h2 mt-4">No published releases yet</h2>
            <p className="t-body mt-3" style={{ color: "var(--ink-2)" }}>
              Tags and release notes will appear here once maintainers publish a
              version.
            </p>
            <Link className="btn mt-5" href={`${basePath(repository)}/tags`}>
              View tags
            </Link>
          </div>
        ) : (
          <>
            <div className="space-y-4">
              {releases.items.map((release) => (
                <ReleaseCard
                  authenticated={authenticated}
                  key={release.id}
                  release={release}
                  repository={repository}
                />
              ))}
            </div>
            <Pagination
              envelope={releases}
              hrefBase={`${basePath(repository)}/releases`}
            />
          </>
        )}
      </section>
    </RepositoryShell>
  );
}

export function RepositoryReleaseDetailPage({
  authenticated,
  release,
  repository,
}: RepositoryReleaseDetailPageProps) {
  return (
    <RepositoryShell
      activePath={`${basePath(repository)}/releases`}
      frameClassName="max-w-5xl"
      repository={repository}
    >
      <section>
        <ReleasesTabs active="releases" repository={repository} />
        {isApiError(release) ? (
          <ReleaseUnavailable error={release} title="Release could not load" />
        ) : (
          <>
            <div className="mb-5 flex flex-wrap items-center justify-between gap-3">
              <Link className="btn" href={`${basePath(repository)}/releases`}>
                Back to releases
              </Link>
              {release.immutable ? (
                <span className="chip soft">Immutable</span>
              ) : null}
            </div>
            <ReleaseCard
              authenticated={authenticated}
              detailHtml={release.bodyHtml}
              release={release}
              repository={repository}
            />
            {release.tagSignatureSummary ? (
              <p className="t-xs mt-3">{release.tagSignatureSummary}</p>
            ) : null}
          </>
        )}
      </section>
    </RepositoryShell>
  );
}

export function RepositoryTagsPage({
  repository,
  tags,
}: RepositoryTagsPageProps) {
  return (
    <RepositoryShell
      activePath={`${basePath(repository)}/tags`}
      frameClassName="max-w-5xl"
      repository={repository}
    >
      <section>
        <ReleasesTabs active="tags" repository={repository} />
        <div className="mb-6">
          <p className="t-label">Repository refs</p>
          <h1 className="t-h1 mt-2">Tags</h1>
        </div>
        {isApiError(tags) ? (
          <ReleaseUnavailable error={tags} title="Tags could not load" />
        ) : tags.items.length === 0 ? (
          <div className="card p-6" role="status">
            <span className="chip soft">No tags</span>
            <h2 className="t-h2 mt-4">No repository tags yet</h2>
            <p className="t-body mt-3" style={{ color: "var(--ink-2)" }}>
              Version tags will appear here after they are pushed to this
              repository.
            </p>
          </div>
        ) : (
          <>
            <div className="card overflow-hidden">
              {tags.items.map((tag) => (
                <div
                  className="list-row flex flex-wrap items-center gap-3 px-4 py-3"
                  key={tag.id}
                >
                  <div className="min-w-0 flex-1">
                    <div className="flex flex-wrap items-center gap-2">
                      <Link
                        className="t-mono hover:underline"
                        href={`${basePath(repository)}/tree/${encodeURIComponent(tag.name)}`}
                      >
                        {tag.name}
                      </Link>
                      {tag.verified ? (
                        <span className="chip ok">Verified</span>
                      ) : null}
                      {tag.releaseHref ? (
                        <Link className="chip accent" href={tag.releaseHref}>
                          Release
                        </Link>
                      ) : null}
                    </div>
                    <p className="t-xs mt-1 truncate">
                      {tag.commitMessage ?? "No commit message recorded."}
                    </p>
                  </div>
                  {tag.shortOid ? (
                    <Link
                      className="t-mono-sm hover:underline"
                      href={`${basePath(repository)}/commit/${tag.targetOid}`}
                    >
                      {tag.shortOid}
                    </Link>
                  ) : null}
                  <span className="t-xs">{formatDate(tag.committedAt)}</span>
                  <Link className="btn sm" href={tag.zipballHref}>
                    Zip
                  </Link>
                  <Link className="btn sm" href={tag.tarballHref}>
                    Tar
                  </Link>
                  <Link
                    className="btn sm"
                    href={compareHref(repository, tag.name)}
                  >
                    Compare
                  </Link>
                </div>
              ))}
            </div>
            <Pagination
              envelope={tags}
              hrefBase={`${basePath(repository)}/tags`}
            />
          </>
        )}
      </section>
    </RepositoryShell>
  );
}

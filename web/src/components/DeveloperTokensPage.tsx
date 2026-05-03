import Link from "next/link";
import { DeveloperCommandBlock } from "@/components/DeveloperCommandBlock";
import type {
  PersonalAccessTokenListFetchResult,
  PersonalAccessTokenSummary,
} from "@/lib/api";

const apiUserExample = `curl -H "Authorization: Bearer <opengithub_pat>" \\
  https://opengithub.namuh.co/api/user`;

const repoListExample = `curl -H "Authorization: Bearer <opengithub_pat>" \\
  "https://opengithub.namuh.co/api/repos?page=1&pageSize=30"`;

const gitCloneExample = `git clone https://mona:<opengithub_pat>@opengithub.namuh.co/mona/octo-app.git`;

const pushExample = `git remote add origin https://opengithub.namuh.co/mona/octo-app.git
git branch -M main
git push -u origin main`;

const scopes = [
  ["repo:read", "Clone, fetch, and read repository metadata."],
  ["repo:write", "Push over HTTPS and mutate repository resources."],
  ["api:read", "Read REST resources through the opengithub API."],
  ["api:write", "Create and update REST resources where permitted."],
];

type DeveloperTokensPageProps = {
  tokenList?: PersonalAccessTokenListFetchResult;
  showHeading?: boolean;
};

export function DeveloperTokensPage({
  tokenList,
  showHeading = true,
}: DeveloperTokensPageProps = {}) {
  return (
    <article className="min-w-0">
      {showHeading ? (
        <div className="pb-5" style={{ borderBottom: "1px solid var(--line)" }}>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Developer settings
          </p>
          <h1 className="mt-2 t-h2">Personal access tokens</h1>
        </div>
      ) : null}
      <p className="max-w-3xl t-body" style={{ color: "var(--ink-3)" }}>
        Use opengithub personal access tokens as command-line credentials for
        Git over HTTPS, REST API calls, and automation. Tokens are stored hashed
        by the Rust API; only prefixes and metadata are shown after creation.
      </p>

      <TokenManagementPanel tokenList={tokenList} />

      <section className="mt-6 card">
        <div className="p-4" style={{ borderBottom: "1px solid var(--line)" }}>
          <h2 className="t-h3">Token quickstart</h2>
          <p className="mt-2 t-body" style={{ color: "var(--ink-3)" }}>
            The examples below use opengithub-owned endpoints and the same token
            contract used by Git transport and REST automation.
          </p>
        </div>
        <div className="grid gap-4 p-4 lg:grid-cols-2">
          <DeveloperCommandBlock
            copyLabel="Copy API curl"
            label="Current user"
            value={apiUserExample}
          />
          <DeveloperCommandBlock
            copyLabel="Copy repo curl"
            label="List repositories"
            value={repoListExample}
          />
          <DeveloperCommandBlock
            copyLabel="Copy clone"
            label="Clone with token"
            value={gitCloneExample}
          />
          <DeveloperCommandBlock
            copyLabel="Copy push"
            label="Push workflow"
            value={pushExample}
          />
        </div>
      </section>

      <section className="mt-6 grid gap-4 lg:grid-cols-2">
        <div className="card p-4">
          <h2 className="t-h3">Recommended scopes</h2>
          <dl className="mt-3 space-y-3 t-sm">
            {scopes.map(([scope, description]) => (
              <div key={scope}>
                <dt
                  className="t-mono-sm font-semibold"
                  style={{ color: "var(--ink-1)" }}
                >
                  {scope}
                </dt>
                <dd
                  className="mt-1 leading-6"
                  style={{ color: "var(--ink-3)" }}
                >
                  {description}
                </dd>
              </div>
            ))}
          </dl>
        </div>
        <div className="card p-4">
          <h2 className="t-h3">Developer references</h2>
          <div className="mt-3 grid gap-2 t-sm">
            <Link
              className="rounded-md px-3 py-2 font-semibold hover:bg-[var(--hover)]"
              style={{
                border: "1px solid var(--line)",
                color: "var(--accent)",
              }}
              href="/docs/git"
            >
              Git over HTTPS guide
            </Link>
            <Link
              className="rounded-md px-3 py-2 font-semibold hover:bg-[var(--hover)]"
              style={{
                border: "1px solid var(--line)",
                color: "var(--accent)",
              }}
              href="/docs/api"
            >
              REST API endpoint catalog
            </Link>
            <Link
              className="rounded-md px-3 py-2 font-semibold hover:bg-[var(--hover)]"
              style={{
                border: "1px solid var(--line)",
                color: "var(--accent)",
              }}
              href="/docs/get-started"
            >
              Setup guide
            </Link>
          </div>
        </div>
      </section>
    </article>
  );
}

function TokenManagementPanel({
  tokenList,
}: {
  tokenList?: PersonalAccessTokenListFetchResult;
}) {
  const tokens = tokenList?.ok ? tokenList.list.tokens : [];
  const sudoActive = tokenList?.ok ? tokenList.list.sudo.active : false;

  return (
    <section className="mt-6 card">
      <div
        className="flex flex-col gap-4 p-4 md:flex-row md:items-start md:justify-between"
        style={{ borderBottom: "1px solid var(--line)" }}
      >
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Credentials
          </p>
          <h2 className="mt-2 t-h3">Your personal access tokens</h2>
          <p
            className="mt-2 max-w-2xl t-body"
            style={{ color: "var(--ink-3)" }}
          >
            Review token prefixes, repository reach, expiration, and last use.
            Token secrets are never returned by this page.
          </p>
        </div>
        <details className="relative">
          <summary className="btn primary cursor-pointer list-none">
            Generate new token
          </summary>
          <div
            className="absolute right-0 z-10 mt-2 w-72 rounded-md p-2"
            style={{
              background: "var(--surface)",
              border: "1px solid var(--line)",
              boxShadow: "var(--shadow-md)",
            }}
          >
            <Link
              className="block rounded-md px-3 py-2 hover:bg-[var(--hover)]"
              href="/settings/personal-access-tokens/new?type=fine_grained"
            >
              <span className="block t-sm font-semibold">
                Fine-grained token
              </span>
              <span className="mt-1 block t-xs">
                Repository-scoped permissions and selected repository access.
              </span>
            </Link>
            <Link
              className="mt-1 block rounded-md px-3 py-2 hover:bg-[var(--hover)]"
              href="/settings/personal-access-tokens/new?type=classic"
            >
              <span className="block t-sm font-semibold">Classic token</span>
              <span className="mt-1 block t-xs">
                Broad scopes for legacy automation and Git credentials.
              </span>
            </Link>
          </div>
        </details>
      </div>

      {tokenList && !tokenList.ok ? (
        <div className="p-4">
          <div
            className="rounded-md p-4"
            style={{ background: "var(--surface-2)" }}
          >
            <p className="t-sm font-semibold" style={{ color: "var(--ink-1)" }}>
              Token settings could not be loaded.
            </p>
            <p className="mt-1 t-sm" style={{ color: "var(--ink-3)" }}>
              {tokenList.status === 401
                ? "Sign in to manage personal access tokens."
                : tokenList.message}
            </p>
            <Link className="btn mt-4" href="/login?next=/settings/tokens">
              Sign in
            </Link>
          </div>
        </div>
      ) : tokens.length > 0 ? (
        <div className="divide-y" style={{ borderColor: "var(--line)" }}>
          {tokens.map((token) => (
            <TokenRow key={token.id} token={token} />
          ))}
        </div>
      ) : (
        <div className="p-4">
          <div
            className="rounded-md p-5"
            style={{ background: "var(--surface-2)" }}
          >
            <h3 className="t-h3">No personal access tokens yet</h3>
            <p
              className="mt-2 max-w-2xl t-body"
              style={{ color: "var(--ink-3)" }}
            >
              Create a fine-grained token when automation needs repository
              access, or use a classic token for older tools that expect broad
              scopes.
            </p>
            <div className="mt-4 flex flex-wrap gap-2">
              <Link
                className="btn primary"
                href="/settings/personal-access-tokens/new?type=fine_grained"
              >
                New fine-grained token
              </Link>
              <Link
                className="btn"
                href="/settings/personal-access-tokens/new?type=classic"
              >
                New classic token
              </Link>
            </div>
          </div>
        </div>
      )}

      <div
        className="flex flex-col gap-2 p-4 t-sm md:flex-row md:items-center md:justify-between"
        style={{ borderTop: "1px solid var(--line)", color: "var(--ink-3)" }}
      >
        <span>
          Sudo mode is{" "}
          {sudoActive ? "active for this session" : "required for token writes"}
          .
        </span>
        <Link
          className="font-semibold"
          href="/docs/api"
          style={{ color: "var(--accent)" }}
        >
          API authentication docs
        </Link>
      </div>
    </section>
  );
}

function TokenRow({ token }: { token: PersonalAccessTokenSummary }) {
  return (
    <article className="list-row p-4">
      <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-2">
            <h3 className="t-h3">{token.name}</h3>
            <span className={statusChipClass(token.status)}>
              {labelize(token.status)}
            </span>
            <span className="chip soft">
              {token.type === "classic" ? "Classic" : "Fine-grained"}
            </span>
          </div>
          {token.description ? (
            <p className="mt-2 t-sm" style={{ color: "var(--ink-3)" }}>
              {token.description}
            </p>
          ) : null}
          <div className="mt-3 flex flex-wrap gap-2">
            {token.scopes.map((scope) => (
              <span className="chip" key={scope}>
                {scope}
              </span>
            ))}
            {token.scopes.length === 0 ? (
              <span className="chip warn">No scopes</span>
            ) : null}
          </div>
        </div>
        <div
          className="grid gap-2 t-sm lg:min-w-72"
          style={{ color: "var(--ink-3)" }}
        >
          <MetaLine label="Prefix" value={token.prefix} mono />
          <MetaLine label="Owner" value={token.resourceOwner.login} />
          <MetaLine label="Repositories" value={repositoryAccessLabel(token)} />
          <MetaLine
            label="Last used"
            value={formatDate(token.lastUsedAt, "Never")}
          />
          <MetaLine
            label="Expires"
            value={formatDate(token.expiresAt, "Never")}
          />
        </div>
      </div>
    </article>
  );
}

function MetaLine({
  label,
  mono = false,
  value,
}: {
  label: string;
  mono?: boolean;
  value: string;
}) {
  return (
    <div className="grid grid-cols-[96px_minmax(0,1fr)] gap-3">
      <span className="t-label" style={{ color: "var(--ink-4)" }}>
        {label}
      </span>
      <span className={mono ? "t-mono-sm truncate" : "truncate"}>{value}</span>
    </div>
  );
}

function statusChipClass(status: string) {
  if (status === "active") {
    return "chip ok";
  }
  if (status === "expired") {
    return "chip warn";
  }
  if (status === "revoked") {
    return "chip err";
  }
  return "chip";
}

function repositoryAccessLabel(token: PersonalAccessTokenSummary) {
  if (token.repositoryAccess === "all") {
    return "All accessible repositories";
  }
  if (token.selectedRepositories.length === 0) {
    return "Selected repositories";
  }
  if (token.selectedRepositories.length === 1) {
    return token.selectedRepositories[0]?.fullName ?? "Selected repositories";
  }
  return `${token.selectedRepositories.length} selected repositories`;
}

function labelize(value: string) {
  return value
    .split("_")
    .filter(Boolean)
    .map((part) => part.slice(0, 1).toUpperCase() + part.slice(1))
    .join(" ");
}

function formatDate(value: string | null, fallback: string) {
  if (!value) {
    return fallback;
  }
  return new Intl.DateTimeFormat("en", {
    day: "numeric",
    month: "short",
    year: "numeric",
  }).format(new Date(value));
}

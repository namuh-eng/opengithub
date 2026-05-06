import { AppShell } from "@/components/AppShell";
import { GlobalPullsPage } from "@/components/GlobalPullsPage";
import type {
  GlobalPullRequestScope,
  PullRequestSort,
  PullRequestState,
} from "@/lib/api";
import {
  getGlobalPullRequests,
  getSessionAndShellContext,
} from "@/lib/server-session";

type PullRequestsPageProps = {
  searchParams: Promise<{
    scope?: GlobalPullRequestScope;
    q?: string;
    state?: PullRequestState;
    repo?: string;
    labels?: string;
    milestone?: string;
    sort?: PullRequestSort;
    page?: string;
  }>;
};

export default async function PullRequestsPage({
  searchParams,
}: PullRequestsPageProps) {
  const [query, { session, shellContext }] = await Promise.all([
    searchParams,
    getSessionAndShellContext(),
  ]);
  const page = Number.parseInt(query.page ?? "1", 10);
  const pullQuery = {
    scope: query.scope,
    q: query.q,
    state: query.state,
    repo: query.repo,
    labels: query.labels
      ?.split(",")
      .map((label) => label.trim())
      .filter(Boolean),
    milestone: query.milestone,
    sort: query.sort,
    page: Number.isFinite(page) ? page : 1,
  };
  const pulls = await getGlobalPullRequests(pullQuery);

  return (
    <AppShell session={session} shellContext={shellContext}>
      <GlobalPullsPage pulls={pulls} />
    </AppShell>
  );
}

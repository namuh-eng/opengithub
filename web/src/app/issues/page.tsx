import { AppShell } from "@/components/AppShell";
import { GlobalIssuesPage } from "@/components/GlobalIssuesPage";
import type { GlobalIssueScope, IssueSort, IssueState } from "@/lib/api";
import {
  getGlobalIssues,
  getSessionAndShellContext,
} from "@/lib/server-session";

type IssuesPageProps = {
  searchParams: Promise<{
    scope?: GlobalIssueScope;
    q?: string;
    state?: IssueState;
    repo?: string;
    labels?: string;
    milestone?: string;
    project?: string;
    sort?: IssueSort;
    page?: string;
  }>;
};

export default async function IssuesPage({ searchParams }: IssuesPageProps) {
  const [query, { session, shellContext }] = await Promise.all([
    searchParams,
    getSessionAndShellContext(),
  ]);
  const page = Number.parseInt(query.page ?? "1", 10);
  const issues = await getGlobalIssues({
    scope: query.scope,
    q: query.q,
    state: query.state,
    repo: query.repo,
    labels: query.labels
      ?.split(",")
      .map((label) => label.trim())
      .filter(Boolean),
    milestone: query.milestone,
    project: query.project,
    sort: query.sort,
    page: Number.isFinite(page) ? page : 1,
  });

  return (
    <AppShell session={session} shellContext={shellContext}>
      <GlobalIssuesPage issues={issues} />
    </AppShell>
  );
}

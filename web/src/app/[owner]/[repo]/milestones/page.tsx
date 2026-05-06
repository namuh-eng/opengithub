import { AppShell } from "@/components/AppShell";
import { RepositoryMilestonesPage as RepositoryMilestonesScreen } from "@/components/RepositoryMilestonesPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import type { MilestoneListState, MilestoneSort } from "@/lib/api";
import {
  getRepository,
  getRepositoryMilestones,
  getSessionAndShellContext,
} from "@/lib/server-session";

type RepositoryMilestonesPageProps = {
  params: Promise<{ owner: string; repo: string }>;
  searchParams: Promise<{
    state?: string;
    sort?: string;
    page?: string;
  }>;
};

const STATES = new Set<MilestoneListState>(["open", "closed", "all"]);
const SORTS = new Set<MilestoneSort>([
  "updated-desc",
  "due-desc",
  "due-asc",
  "complete-asc",
  "complete-desc",
  "alpha-asc",
  "alpha-desc",
  "issues-desc",
  "issues-asc",
]);

export default async function RepositoryMilestonesPage({
  params,
  searchParams,
}: RepositoryMilestonesPageProps) {
  const [{ owner, repo }, query, { session, shellContext }] = await Promise.all(
    [params, searchParams, getSessionAndShellContext()],
  );
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const page = Number.parseInt(query.page ?? "1", 10);
  const requestedState = query.state as MilestoneListState | undefined;
  const requestedSort = query.sort as MilestoneSort | undefined;
  const milestonesQuery = {
    state:
      requestedState && STATES.has(requestedState) ? requestedState : "open",
    sort:
      requestedSort && SORTS.has(requestedSort)
        ? requestedSort
        : "updated-desc",
    page: Number.isFinite(page) ? page : 1,
    pageSize: 100,
  };
  const [repository, milestones] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryMilestones(ownerLogin, repositoryName, milestonesQuery).catch(
      () => null,
    ),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository && milestones && !("error" in milestones) ? (
        <RepositoryMilestonesScreen
          milestones={milestones}
          query={milestonesQuery}
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}

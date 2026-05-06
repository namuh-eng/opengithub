import { AppShell } from "@/components/AppShell";
import { RepositoryMilestoneDetailPage as RepositoryMilestoneDetailScreen } from "@/components/RepositoryMilestoneDetailPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryMilestone,
  getSessionAndShellContext,
} from "@/lib/server-session";

type RepositoryMilestonePageProps = {
  params: Promise<{ owner: string; repo: string; milestone: string }>;
  searchParams: Promise<{ state?: string }>;
};

export default async function RepositoryMilestonePage({
  params,
  searchParams,
}: RepositoryMilestonePageProps) {
  const [{ owner, repo, milestone }, query, { session, shellContext }] =
    await Promise.all([params, searchParams, getSessionAndShellContext()]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const milestoneId = decodeURIComponent(milestone);
  const [repository, milestoneDetail] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryMilestone(ownerLogin, repositoryName, milestoneId).catch(
      () => null,
    ),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository && milestoneDetail && !("error" in milestoneDetail) ? (
        <RepositoryMilestoneDetailScreen
          milestone={milestoneDetail}
          query={{ state: query.state ?? null }}
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}

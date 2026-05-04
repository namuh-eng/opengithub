import { AppShell } from "@/components/AppShell";
import { RepositoryBranchActivityPage } from "@/components/RepositoryBranchActivityPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryBranchActivity,
  getSession,
} from "@/lib/server-session";

type BranchActivityRouteProps = {
  params: Promise<{ owner: string; repo: string; branch: string[] }>;
};

export default async function BranchActivityRoute({
  params,
}: BranchActivityRouteProps) {
  const [{ owner, repo, branch }, session] = await Promise.all([
    params,
    getSession(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const branchName = branch.map(decodeURIComponent).join("/");
  const [repository, activityResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryBranchActivity(ownerLogin, repositoryName, branchName),
  ]);

  return (
    <AppShell session={session}>
      {repository ? (
        <RepositoryBranchActivityPage
          activityResult={activityResult}
          branchName={branchName}
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}

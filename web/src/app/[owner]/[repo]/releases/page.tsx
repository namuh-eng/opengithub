import { AppShell } from "@/components/AppShell";
import { RepositoryReleasesPage } from "@/components/RepositoryReleasesPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryReleases,
  getSession,
} from "@/lib/server-session";

type ReleasesPageProps = {
  params: Promise<{ owner: string; repo: string }>;
  searchParams?: Promise<{ page?: string }>;
};

export default async function ReleasesPage({
  params,
  searchParams,
}: ReleasesPageProps) {
  const [{ owner, repo }, query, session] = await Promise.all([
    params,
    searchParams,
    getSession(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const repository = await getRepository(ownerLogin, repositoryName);
  const page = Number.parseInt(query?.page ?? "1", 10);
  const releases = repository
    ? await getRepositoryReleases(ownerLogin, repositoryName, {
        page: Number.isFinite(page) && page > 0 ? page : 1,
      })
    : null;
  return (
    <AppShell session={session}>
      {repository && releases ? (
        <RepositoryReleasesPage releases={releases} repository={repository} />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}

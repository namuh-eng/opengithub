import { AppShell } from "@/components/AppShell";
import { RepositoryTagsPage } from "@/components/RepositoryReleasesPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryReleaseTags,
  getSession,
} from "@/lib/server-session";

type TagsPageProps = {
  params: Promise<{ owner: string; repo: string }>;
  searchParams?: Promise<{ page?: string }>;
};

export default async function TagsPage({
  params,
  searchParams,
}: TagsPageProps) {
  const [{ owner, repo }, query, session] = await Promise.all([
    params,
    searchParams,
    getSession(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const repository = await getRepository(ownerLogin, repositoryName);
  const page = Number.parseInt(query?.page ?? "1", 10);
  const tags = repository
    ? await getRepositoryReleaseTags(ownerLogin, repositoryName, {
        page: Number.isFinite(page) && page > 0 ? page : 1,
      })
    : null;
  return (
    <AppShell session={session}>
      {repository && tags ? (
        <RepositoryTagsPage repository={repository} tags={tags} />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}

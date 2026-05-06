import { SocialListPage } from "@/components/SocialListPage";
import { getRepositoryStargazers, getSession } from "@/lib/server-session";

type PageProps = {
  params: Promise<{ owner: string; repo: string }>;
  searchParams?: Promise<{ page?: string; pageSize?: string }>;
};

function positive(value: string | undefined) {
  const parsed = Number.parseInt(value ?? "", 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : undefined;
}

export default async function StargazersPage({
  params,
  searchParams,
}: PageProps) {
  const [{ owner, repo }, query, session] = await Promise.all([
    params,
    searchParams,
    getSession(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const list = await getRepositoryStargazers(ownerLogin, repositoryName, {
    page: positive(query?.page),
    pageSize: positive(query?.pageSize),
  });

  return (
    <SocialListPage
      backHref={`/${encodeURIComponent(ownerLogin)}/${encodeURIComponent(repositoryName)}`}
      backLabel="Back to repository"
      empty={`${ownerLogin}/${repositoryName} does not have visible stars yet.`}
      eyebrow="Repository stars"
      list={list}
      session={session}
      title={`${ownerLogin}/${repositoryName} stargazers`}
    />
  );
}

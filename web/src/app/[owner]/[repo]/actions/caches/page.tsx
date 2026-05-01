import { RepositoryFeaturePage } from "@/components/RepositoryFeaturePage";

type ActionsCachesPageProps = {
  params: Promise<{ owner: string; repo: string }>;
};

export default async function ActionsCachesPage({
  params,
}: ActionsCachesPageProps) {
  const { owner, repo } = await params;
  const base = `/${decodeURIComponent(owner)}/${decodeURIComponent(repo)}`;

  return (
    <RepositoryFeaturePage
      actions={[
        { href: `${base}/actions`, label: "All workflows", primary: true },
        { href: "/docs/api#actions-dashboard", label: "Actions API docs" },
      ]}
      activePath={`${base}/actions/caches`}
      description="Workflow cache entries will be listed here with retention, size, and delete controls after runner execution starts writing cache metadata."
      owner={owner}
      repo={repo}
      title="Actions caches"
    />
  );
}

import { RepositoryFeaturePage } from "@/components/RepositoryFeaturePage";

type ActionsUsagePageProps = {
  params: Promise<{ owner: string; repo: string }>;
};

export default async function ActionsUsagePage({
  params,
}: ActionsUsagePageProps) {
  const { owner, repo } = await params;
  const base = `/${decodeURIComponent(owner)}/${decodeURIComponent(repo)}`;

  return (
    <RepositoryFeaturePage
      actions={[
        { href: `${base}/actions`, label: "All workflows", primary: true },
        { href: "/docs/api#actions-dashboard", label: "Actions API docs" },
      ]}
      activePath={`${base}/actions/usage`}
      description="Usage metrics will summarize workflow minutes, storage, and retention once background runner accounting is implemented."
      owner={owner}
      repo={repo}
      title="Actions usage metrics"
    />
  );
}

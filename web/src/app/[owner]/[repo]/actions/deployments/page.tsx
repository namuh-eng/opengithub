import { RepositoryFeaturePage } from "@/components/RepositoryFeaturePage";

type ActionsDeploymentsPageProps = {
  params: Promise<{ owner: string; repo: string }>;
};

export default async function ActionsDeploymentsPage({
  params,
}: ActionsDeploymentsPageProps) {
  const { owner, repo } = await params;
  const base = `/${decodeURIComponent(owner)}/${decodeURIComponent(repo)}`;

  return (
    <RepositoryFeaturePage
      actions={[
        { href: `${base}/actions`, label: "All workflows", primary: true },
        { href: `${base}/settings/actions`, label: "Actions policy" },
      ]}
      activePath={`${base}/actions/deployments`}
      description="Deployment records, environment gates, and workflow-created releases will appear here when deployment events are implemented."
      owner={owner}
      repo={repo}
      title="Actions deployments"
    />
  );
}

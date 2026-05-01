import { RepositoryFeaturePage } from "@/components/RepositoryFeaturePage";

type ActionsAttestationsPageProps = {
  params: Promise<{ owner: string; repo: string }>;
};

export default async function ActionsAttestationsPage({
  params,
}: ActionsAttestationsPageProps) {
  const { owner, repo } = await params;
  const base = `/${decodeURIComponent(owner)}/${decodeURIComponent(repo)}`;

  return (
    <RepositoryFeaturePage
      actions={[
        { href: `${base}/actions`, label: "All workflows", primary: true },
        { href: "/docs/api#actions-runs", label: "Workflow run API" },
      ]}
      activePath={`${base}/actions/attestations`}
      description="Artifact attestations and provenance checks will be reviewed here after artifact storage and verification are connected."
      owner={owner}
      repo={repo}
      title="Artifact attestations"
    />
  );
}

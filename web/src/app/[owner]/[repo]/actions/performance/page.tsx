import { RepositoryFeaturePage } from "@/components/RepositoryFeaturePage";

type ActionsPerformancePageProps = {
  params: Promise<{ owner: string; repo: string }>;
};

export default async function ActionsPerformancePage({
  params,
}: ActionsPerformancePageProps) {
  const { owner, repo } = await params;
  const base = `/${decodeURIComponent(owner)}/${decodeURIComponent(repo)}`;

  return (
    <RepositoryFeaturePage
      actions={[
        { href: `${base}/actions`, label: "All workflows", primary: true },
        { href: `${base}/actions/usage`, label: "Usage metrics" },
      ]}
      activePath={`${base}/actions/performance`}
      description="Performance metrics will show queue time, slow jobs, and flake signals after workflow job telemetry is collected."
      owner={owner}
      repo={repo}
      title="Actions performance metrics"
    />
  );
}

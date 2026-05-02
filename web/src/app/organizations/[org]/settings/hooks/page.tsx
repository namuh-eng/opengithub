import { AppShell } from "@/components/AppShell";
import { WebhooksSettingsPage } from "@/components/WebhooksSettingsPage";
import {
  getOrganizationWebhooks,
  getSessionAndShellContext,
} from "@/lib/server-session";

type OrganizationHooksPageProps = { params: Promise<{ org: string }> };

export default async function OrganizationHooksPage({
  params,
}: OrganizationHooksPageProps) {
  const { org } = await params;
  const orgLogin = decodeURIComponent(org);
  const [{ session, shellContext }, catalog] = await Promise.all([
    getSessionAndShellContext(),
    getOrganizationWebhooks(orgLogin),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      <main className="mx-auto w-full max-w-[1240px] px-6 py-8">
        <div className="pb-5" style={{ borderBottom: "1px solid var(--line)" }}>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Organization settings
          </p>
          <h1 className="t-h1 mt-2">Webhooks</h1>
        </div>
        <div className="mt-6">
          <WebhooksSettingsPage
            catalog={catalog}
            endpointBase={`/api/orgs/${encodeURIComponent(orgLogin)}/hooks`}
            ownerLabel={orgLogin}
          />
        </div>
      </main>
    </AppShell>
  );
}

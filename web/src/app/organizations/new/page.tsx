import { AppShell } from "@/components/AppShell";
import { OrganizationCreatePage } from "@/components/OrganizationCreatePage";
import { getSessionAndShellContext } from "@/lib/server-session";

export default async function NewOrganizationPage() {
  const { session, shellContext } = await getSessionAndShellContext();

  return (
    <AppShell session={session} shellContext={shellContext}>
      <OrganizationCreatePage />
    </AppShell>
  );
}

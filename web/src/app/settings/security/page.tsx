import { AccountSecurityPage } from "@/components/AccountSecurityPage";
import { SettingsShell } from "@/components/SettingsShell";
import {
  getAccountSecuritySettings,
  getSessionAndShellContext,
} from "@/lib/server-session";

export default async function SecuritySettingsPage() {
  const [{ session, shellContext }, securitySettings] = await Promise.all([
    getSessionAndShellContext(),
    getAccountSecuritySettings(),
  ]);
  const apiUrl = process.env.API_URL ?? "http://localhost:3016";
  const linkGoogleHref = `${apiUrl}/api/settings/security/google/link?next=/settings/security`;

  return (
    <SettingsShell
      activeSection="security"
      eyebrow="Personal settings"
      session={session}
      shellContext={shellContext}
      title="Security"
    >
      <AccountSecurityPage
        linkGoogleHref={linkGoogleHref}
        securitySettings={securitySettings}
        userEmail={session.user?.email}
      />
    </SettingsShell>
  );
}

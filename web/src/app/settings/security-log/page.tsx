import { AccountSecurityLogPage as AccountSecurityLogView } from "@/components/AccountSecurityLogPage";
import { SettingsShell } from "@/components/SettingsShell";
import {
  getAccountSecurityLog,
  getSessionAndShellContext,
} from "@/lib/server-session";

type SecurityLogPageProps = {
  searchParams?: Promise<Record<string, string | string[] | undefined>>;
};

export default async function SecurityLogPage({
  searchParams,
}: SecurityLogPageProps) {
  const params = (await searchParams) ?? {};
  const action = firstParam(params.action) ?? null;
  const page = Math.max(1, Number(firstParam(params.page) ?? "1") || 1);
  const [{ session, shellContext }, logResult] = await Promise.all([
    getSessionAndShellContext(),
    getAccountSecurityLog({ action, page }),
  ]);

  return (
    <SettingsShell
      activeSection="security-log"
      eyebrow="Personal settings"
      session={session}
      shellContext={shellContext}
      title="Security log"
    >
      <AccountSecurityLogView
        action={action}
        logResult={logResult}
        page={page}
      />
    </SettingsShell>
  );
}

function firstParam(value: string | string[] | undefined) {
  return Array.isArray(value) ? value[0] : value;
}

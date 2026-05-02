import { AppearanceSettingsForm } from "@/components/AppearanceSettingsForm";
import { SettingsShell } from "@/components/SettingsShell";
import {
  getSessionAndShellContext,
  getUserAppearanceSettings,
} from "@/lib/server-session";

const fallbackSettings = {
  userId: "pending",
  theme: "system" as const,
  fontSize: "medium" as const,
  updatedAt: new Date(0).toISOString(),
};

export default async function AppearanceSettingsPage() {
  const [{ session, shellContext }, settings] = await Promise.all([
    getSessionAndShellContext(),
    getUserAppearanceSettings(),
  ]);

  return (
    <SettingsShell
      activeSection="appearance"
      eyebrow="Settings"
      session={session}
      shellContext={shellContext}
      title="Appearance"
    >
      <AppearanceSettingsForm initialSettings={settings ?? fallbackSettings} />
    </SettingsShell>
  );
}

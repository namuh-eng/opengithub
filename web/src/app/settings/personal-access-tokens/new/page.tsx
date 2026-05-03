import { PersonalAccessTokenCreatePage } from "@/components/PersonalAccessTokenCreatePage";
import { SettingsShell } from "@/components/SettingsShell";
import {
  getPersonalAccessTokenNewContext,
  getSessionAndShellContext,
} from "@/lib/server-session";

type NewPersonalAccessTokenPageProps = {
  searchParams?: Promise<Record<string, string | string[] | undefined>>;
};

export default async function NewPersonalAccessTokenPage({
  searchParams,
}: NewPersonalAccessTokenPageProps) {
  const params = (await searchParams) ?? {};
  const [{ session, shellContext }, contextResult] = await Promise.all([
    getSessionAndShellContext(),
    getPersonalAccessTokenNewContext(),
  ]);

  return (
    <SettingsShell
      activeSection="tokens"
      eyebrow="Developer settings"
      session={session}
      shellContext={shellContext}
      title="New fine-grained token"
    >
      <PersonalAccessTokenCreatePage
        contextResult={contextResult}
        initialQuery={{
          api: firstParam(params.api),
          contents: firstParam(params.contents),
          description: firstParam(params.description),
          expires_in: firstParam(params.expires_in),
          issues: firstParam(params.issues),
          name: firstParam(params.name),
          packages: firstParam(params.packages),
          profile: firstParam(params.profile),
          pull_requests: firstParam(params.pull_requests),
          target_name: firstParam(params.target_name),
          type: firstParam(params.type),
        }}
        userEmail={session.user?.email ?? null}
      />
    </SettingsShell>
  );
}

function firstParam(value: string | string[] | undefined) {
  return Array.isArray(value) ? value[0] : value;
}

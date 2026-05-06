import { ProfileOrgShell } from "@/components/ProfileOrgShell";
import { SocialListPage } from "@/components/SocialListPage";
import { PROFILE_TABS, profileTabHref } from "@/lib/navigation";
import {
  getProfileSocialList,
  getSessionAndShellContext,
} from "@/lib/server-session";

type PageProps = {
  params: Promise<{ owner: string }>;
  searchParams?: Promise<{ page?: string; pageSize?: string }>;
};

function positive(value: string | undefined) {
  const parsed = Number.parseInt(value ?? "", 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : undefined;
}

export default async function FollowingPage({
  params,
  searchParams,
}: PageProps) {
  const [{ owner }, query, { session, shellContext }] = await Promise.all([
    params,
    searchParams,
    getSessionAndShellContext(),
  ]);
  const login = decodeURIComponent(owner);
  const list = await getProfileSocialList(login, "following", {
    page: positive(query?.page),
    pageSize: positive(query?.pageSize),
  });

  if (!list) {
    return (
      <ProfileOrgShell
        activeTab="overview"
        eyebrow="Following"
        hrefForTab={(value) => profileTabHref(login, value)}
        identityLabel={login}
        message={`Following for ${login} is unavailable.`}
        session={session}
        shellContext={shellContext}
        tabLabel="Profile sections"
        tabs={PROFILE_TABS}
        title={login}
      />
    );
  }

  return (
    <SocialListPage
      backHref={`/${encodeURIComponent(login)}`}
      backLabel="Back to profile"
      empty={`${login} is not visibly following anyone yet.`}
      eyebrow="Profile social graph"
      list={list}
      session={session}
      title={`${login} following`}
    />
  );
}

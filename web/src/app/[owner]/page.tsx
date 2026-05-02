import { ProfileOrgShell } from "@/components/ProfileOrgShell";
import { UserProfilePage } from "@/components/UserProfilePage";
import {
  activeProfileTab,
  PROFILE_TABS,
  profileTabHref,
} from "@/lib/navigation";
import {
  getProfileRepositories,
  getProfileStars,
  getPublicUserProfile,
  getSessionAndShellContext,
  getUserPackages,
} from "@/lib/server-session";

type ProfilePageProps = {
  params: Promise<{ owner: string }>;
  searchParams?: Promise<Record<string, string | string[] | undefined>>;
};

function firstParam(value: string | string[] | undefined) {
  return Array.isArray(value) ? value[0] : value;
}

function numericYear(value: string | string[] | undefined) {
  const first = firstParam(value);
  if (!first) {
    return undefined;
  }
  const year = Number.parseInt(first, 10);
  return Number.isFinite(year) ? year : undefined;
}

function numericPositive(value: string | string[] | undefined) {
  const first = firstParam(value);
  if (!first) {
    return undefined;
  }
  const number = Number.parseInt(first, 10);
  return Number.isFinite(number) && number > 0 ? number : undefined;
}

export default async function ProfilePage({
  params,
  searchParams,
}: ProfilePageProps) {
  const [{ owner }, queryParams, { session, shellContext }] = await Promise.all(
    [params, searchParams, getSessionAndShellContext()],
  );
  const ownerLogin = decodeURIComponent(owner);
  const activeTab = activeProfileTab(firstParam(queryParams?.tab));
  const profile = await getPublicUserProfile(ownerLogin, {
    year: numericYear(queryParams?.year),
  });
  const [repositoryList, packageList] = await Promise.all([
    activeTab === "repositories" || activeTab === "stars"
      ? (activeTab === "stars" ? getProfileStars : getProfileRepositories)(
          ownerLogin,
          {
            q: firstParam(queryParams?.q),
            type:
              activeTab === "repositories"
                ? firstParam(queryParams?.type)
                : undefined,
            language: firstParam(queryParams?.language),
            sort: firstParam(queryParams?.sort),
            page: numericPositive(queryParams?.page),
            pageSize: numericPositive(queryParams?.pageSize),
          },
        )
      : Promise.resolve(null),
    activeTab === "packages"
      ? getUserPackages(ownerLogin, {
          q: firstParam(queryParams?.q),
          type: firstParam(queryParams?.type),
          visibility: firstParam(queryParams?.visibility),
          sort: firstParam(queryParams?.sort),
          artifactTab: firstParam(queryParams?.artifactTab),
          page: numericPositive(queryParams?.page),
          pageSize: numericPositive(queryParams?.pageSize),
        })
      : Promise.resolve(null),
  ]);

  if (profile) {
    return (
      <UserProfilePage
        activeTab={activeTab}
        profile={profile}
        packageList={packageList}
        repositoryList={repositoryList}
        session={session}
        shellContext={shellContext}
      />
    );
  }

  const activeTabLabel =
    PROFILE_TABS.find((tab) => tab.value === activeTab)?.label ?? "Overview";

  return (
    <ProfileOrgShell
      activeTab={activeTab}
      eyebrow="Profile"
      hrefForTab={(value) => profileTabHref(ownerLogin, value)}
      identityLabel={ownerLogin}
      message={`${activeTabLabel} for ${ownerLogin} is unavailable. The profile may not exist yet, or the profile API could not be reached.`}
      session={session}
      shellContext={shellContext}
      tabLabel="Profile sections"
      tabs={PROFILE_TABS}
      title={ownerLogin}
    />
  );
}

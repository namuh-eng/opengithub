import { notFound } from "next/navigation";
import { UserProfileOverview } from "@/components/UserProfileOverview";
import type { ApiErrorEnvelope } from "@/lib/api";
import {
  activeProfileTab,
  PROFILE_TABS,
  profileTabHref,
} from "@/lib/navigation";
import {
  getSessionAndShellContext,
  getUserProfile,
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
  if (!first) return undefined;
  const parsed = Number.parseInt(first, 10);
  return Number.isFinite(parsed) && parsed >= 2005 && parsed <= 2100
    ? parsed
    : undefined;
}

function isApiError(value: unknown): value is ApiErrorEnvelope {
  return (
    typeof value === "object" &&
    value !== null &&
    "error" in value &&
    "status" in value
  );
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
  const profile = await getUserProfile(
    ownerLogin,
    numericYear(queryParams?.year),
  );

  if (isApiError(profile)) {
    if (profile.status === 404) {
      notFound();
    }
    throw new Error(profile.error.message);
  }

  return (
    <UserProfileOverview
      activeTab={activeTab}
      hrefForTab={(value) => profileTabHref(ownerLogin, value)}
      profile={profile}
      session={session}
      shellContext={shellContext}
      tabs={PROFILE_TABS}
    />
  );
}

import { AppShell } from "@/components/AppShell";
import { AppShellFrame } from "@/components/AppShellFrame";
import { PackageDetailPage } from "@/components/PackageDetailPage";
import {
  getSessionAndShellContext,
  getUserPackageDetail,
} from "@/lib/server-session";

type UserPackageDetailRouteProps = {
  params: Promise<{
    owner: string;
    repo: string;
    package_name: string;
  }>;
  searchParams?: Promise<Record<string, string | string[] | undefined>>;
};

function firstParam(value: string | string[] | undefined) {
  return Array.isArray(value) ? value[0] : value;
}

export default async function UserPackageDetailRoute({
  params,
  searchParams,
}: UserPackageDetailRouteProps) {
  const [{ owner, repo, package_name }, queryParams, shell] = await Promise.all(
    [params, searchParams, getSessionAndShellContext()],
  );
  const ownerLogin = decodeURIComponent(owner);
  const packageType = decodeURIComponent(repo);
  const packageName = decodeURIComponent(package_name);
  const result = await getUserPackageDetail(
    ownerLogin,
    packageType,
    packageName,
    firstParam(queryParams?.version),
  );

  return (
    <AppShell session={shell.session} shellContext={shell.shellContext}>
      <AppShellFrame>
        <PackageDetailPage
          owner={ownerLogin}
          ownerKind="user"
          result={result}
        />
      </AppShellFrame>
    </AppShell>
  );
}

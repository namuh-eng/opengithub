import { AppShell } from "@/components/AppShell";
import { AppShellFrame } from "@/components/AppShellFrame";
import { PackageDetailPage } from "@/components/PackageDetailPage";
import {
  getOrganizationPackageDetail,
  getSessionAndShellContext,
} from "@/lib/server-session";

type OrganizationPackageDetailRouteProps = {
  params: Promise<{
    org: string;
    package_type: string;
    package_name: string;
  }>;
  searchParams?: Promise<Record<string, string | string[] | undefined>>;
};

function firstParam(value: string | string[] | undefined) {
  return Array.isArray(value) ? value[0] : value;
}

export default async function OrganizationPackageDetailRoute({
  params,
  searchParams,
}: OrganizationPackageDetailRouteProps) {
  const [{ org, package_type, package_name }, queryParams, shell] =
    await Promise.all([params, searchParams, getSessionAndShellContext()]);
  const orgLogin = decodeURIComponent(org);
  const packageType = decodeURIComponent(package_type);
  const packageName = decodeURIComponent(package_name);
  const result = await getOrganizationPackageDetail(
    orgLogin,
    packageType,
    packageName,
    firstParam(queryParams?.version),
  );

  return (
    <AppShell session={shell.session} shellContext={shell.shellContext}>
      <AppShellFrame>
        <PackageDetailPage
          owner={orgLogin}
          ownerKind="organization"
          result={result}
        />
      </AppShellFrame>
    </AppShell>
  );
}

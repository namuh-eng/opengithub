import { PackageSettingsPage } from "@/components/PackageSettingsPage";
import { getOrganizationPackageSettings } from "@/lib/server-session";

type OrganizationPackageSettingsRouteProps = {
  params: Promise<{
    org: string;
    package_type: string;
    package_name: string;
  }>;
};

export default async function OrganizationPackageSettingsRoute({
  params,
}: OrganizationPackageSettingsRouteProps) {
  const {
    org,
    package_type: packageType,
    package_name: packageName,
  } = await params;
  const result = await getOrganizationPackageSettings(
    org,
    packageType,
    packageName,
  );

  return (
    <main className="mx-auto w-full max-w-[1240px] px-4 py-8 sm:px-6 lg:px-8">
      <PackageSettingsPage
        owner={org}
        ownerKind="organization"
        result={result}
      />
    </main>
  );
}

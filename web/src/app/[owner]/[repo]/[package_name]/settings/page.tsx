import { PackageSettingsPage } from "@/components/PackageSettingsPage";
import { getUserPackageSettings } from "@/lib/server-session";

type UserPackageSettingsRouteProps = {
  params: Promise<{
    owner: string;
    repo: string;
    package_name: string;
  }>;
};

export default async function UserPackageSettingsRoute({
  params,
}: UserPackageSettingsRouteProps) {
  const { owner, repo: packageType, package_name: packageName } = await params;
  const result = await getUserPackageSettings(owner, packageType, packageName);

  return (
    <main className="mx-auto w-full max-w-[1240px] px-4 py-8 sm:px-6 lg:px-8">
      <PackageSettingsPage owner={owner} ownerKind="user" result={result} />
    </main>
  );
}

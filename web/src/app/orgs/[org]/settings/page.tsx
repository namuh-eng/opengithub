import { redirect } from "next/navigation";

type OrganizationSettingsPageProps = {
  params: Promise<{ org: string }>;
};

export default async function OrganizationSettingsPage({
  params,
}: OrganizationSettingsPageProps) {
  const { org } = await params;
  redirect(`/organizations/${encodeURIComponent(org)}/settings/profile`);
}

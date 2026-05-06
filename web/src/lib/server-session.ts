import { headers } from "next/headers";
import {
  type AccountSecuritySettingsFetchResult,
  type CodeSearchQuery,
  type CollaborationSearchQuery,
  type DashboardSummaryQuery,
  type GlobalSearchQuery,
  getAccountSecuritySettingsFromCookie,
  getAppShellContextFromCookie,
  getDashboardSummaryFromCookie,
  getKeySettingsFromCookie,
  getNotificationDeliverySettingsFromCookie,
  getNotificationFilterSettingsFromCookie,
  getOrganizationMemberPrivilegesFromCookie,
  getOrganizationPackageDetailFromCookie,
  getOrganizationPackageSettingsFromCookie,
  getOrganizationPackagesFromCookie,
  getOrganizationPeopleAdminFromCookie,
  getOrganizationPeopleFromCookie,
  getOrganizationProfileSettingsFromCookie,
  getOrganizationProjectFieldSettingsFromCookie,
  getOrganizationProjectsFromCookie,
  getOrganizationProjectWorkspaceFromCookie,
  getOrganizationRepositoriesFromCookie,
  getOrganizationTeamDetailFromCookie,
  getOrganizationTeamsFromCookie,
  getPersonalAccessTokenListFromCookie,
  getPersonalAccessTokenNewContextFromCookie,
  getPersonalProfileSettingsFromCookie,
  getProfileRepositoriesFromCookie,
  getProfileStarsFromCookie,
  getProjectItemDetailFromCookie,
  getPublicOrganizationProfileFromCookie,
  getPublicUserProfileFromCookie,
  getPullRequestCompareFromCookie,
  getRepositoryAccessSettingsFromCookie,
  getRepositoryActionsDashboardFromCookie,
  getRepositoryActionsJobLogDetailFromCookie,
  getRepositoryActionsRunDetailFromCookie,
  getRepositoryActionsSecretsSettingsFromCookie,
  getRepositoryActionsWorkflowDashboardFromCookie,
  getRepositoryBlameFromCookie,
  getRepositoryBlobFromCookie,
  getRepositoryBranchActivityFromCookie,
  getRepositoryBranchesFromCookie,
  getRepositoryBranchSettingsFromCookie,
  getRepositoryCodeScanningAlertDetailFromCookie,
  getRepositoryCodeScanningAlertsFromCookie,
  getRepositoryCommitDetailFromCookie,
  getRepositoryCommitHistoryFromCookie,
  getRepositoryContributorsFromCookie,
  getRepositoryCreationOptionsFromCookie,
  getRepositoryDependabotAlertDetailFromCookie,
  getRepositoryDependabotAlertsFromCookie,
  getRepositoryDependenciesFromCookie,
  getRepositoryDependentsFromCookie,
  getRepositoryDiscussionCategorySettingsFromCookie,
  getRepositoryDiscussionCategoryTemplateFromCookie,
  getRepositoryDiscussionCreationFromCookie,
  getRepositoryDiscussionDetailFromCookie,
  getRepositoryDiscussionsFromCookie,
  getRepositoryFileFinderFromCookie,
  getRepositoryForksFromCookie,
  getRepositoryFromCookie,
  getRepositoryImportFromCookie,
  getRepositoryIssueFromCookie,
  getRepositoryIssuesFromCookie,
  getRepositoryIssueTemplatesFromCookie,
  getRepositoryIssueTimelineFromCookie,
  getRepositoryNetworkFromCookie,
  getRepositoryPagesSettingsFromCookie,
  getRepositoryPathFromCookie,
  getRepositoryProjectsFromCookie,
  getRepositoryPullRequestFilesFromCookie,
  getRepositoryPullRequestFromCookie,
  getRepositoryPullRequestsFromCookie,
  getRepositoryPullRequestTimelineFromCookie,
  getRepositoryPulseFromCookie,
  getRepositoryRefsFromCookie,
  getRepositoryReleaseDetailFromCookie,
  getRepositoryReleaseManagementContextFromCookie,
  getRepositoryReleasesFromCookie,
  getRepositoryReleaseTagsFromCookie,
  getRepositorySecretScanningAlertDetailFromCookie,
  getRepositorySecretScanningAlertsFromCookie,
  getRepositorySecurityAdvisoriesFromCookie,
  getRepositorySecurityAdvisoryDetailFromCookie,
  getRepositorySecurityOverviewFromCookie,
  getRepositorySecurityPolicyFromCookie,
  getRepositorySettingsFromCookie,
  getRepositoryTrafficFromCookie,
  getRepositoryWebhookDeliveryDetailFromCookie,
  getRepositoryWebhookDetailFromCookie,
  getRepositoryWebhookSettingsFromCookie,
  getSearchSuggestionsFromCookie,
  getSessionFromHeaders,
  getUserPackageDetailFromCookie,
  getUserPackageSettingsFromCookie,
  getUserPackagesFromCookie,
  getUserProjectFieldSettingsFromCookie,
  getUserProjectsFromCookie,
  getUserProjectWorkspaceFromCookie,
  type KeySettingsFetchResult,
  type OrganizationMemberPrivilegesFetchResult,
  type OrganizationPeopleAdminQuery,
  type OrganizationPeopleListQuery,
  type OrganizationProfileSettingsFetchResult,
  type OrganizationRepositoryListQuery,
  type OrganizationTeamsQuery,
  type OwnerPackageListQuery,
  type PackageDetailFetchResult,
  type PackageSettingsFetchResult,
  type PersonalAccessTokenListFetchResult,
  type PersonalAccessTokenNewContextFetchResult,
  type ProfileRepositoryListQuery,
  type ProjectFieldSettingsFetchResult,
  type ProjectItemDetailFetchResult,
  type ProjectListFetchResult,
  type ProjectListQuery,
  type ProjectWorkspaceFetchResult,
  type ProjectWorkspaceQuery,
  type RepositoryActionsDashboardQuery,
  type RepositoryBranchActivityFetchResult,
  type RepositoryBranchesFetchResult,
  type RepositoryCodeScanningAlertDetailFetchResult,
  type RepositoryCodeScanningAlertsFetchResult,
  type RepositoryCodeScanningAlertsQuery,
  type RepositoryContributorsFetchResult,
  type RepositoryDependabotAlertDetailFetchResult,
  type RepositoryDependabotAlertsFetchResult,
  type RepositoryDependabotAlertsQuery,
  type RepositoryDependenciesFetchResult,
  type RepositoryDependenciesQuery,
  type RepositoryDependentsFetchResult,
  type RepositoryDependentsQuery,
  type RepositoryDiscussionCreationQuery,
  type RepositoryDiscussionDetailQuery,
  type RepositoryDiscussionsQuery,
  type RepositoryForksFetchResult,
  type RepositoryForksQuery,
  type RepositoryIssueListQuery,
  type RepositoryNetworkFetchResult,
  type RepositoryPullRequestDiffQuery,
  type RepositoryPullRequestListQuery,
  type RepositoryPulseFetchResult,
  type RepositoryReleaseListQuery,
  type RepositorySecretScanningAlertDetailFetchResult,
  type RepositorySecretScanningAlertsFetchResult,
  type RepositorySecretScanningAlertsQuery,
  type RepositorySecurityAdvisoriesFetchResult,
  type RepositorySecurityAdvisoriesQuery,
  type RepositorySecurityAdvisoryDetailFetchResult,
  type RepositorySecurityOverviewFetchResult,
  type RepositorySecurityPolicyFetchResult,
  type RepositoryTrafficFetchResult,
  type SearchSuggestionsQuery,
  searchCodeFromCookie,
  searchCollaborationFromCookie,
  searchGlobalFromCookie,
} from "@/lib/api";

export async function getSession() {
  return getSessionFromHeaders(await headers());
}

export async function getSessionAndShellContext() {
  const requestHeaders = await headers();
  const cookie = requestHeaders.get("cookie");
  const [session, shellContext] = await Promise.all([
    getSessionFromHeaders(requestHeaders),
    getAppShellContextFromCookie(cookie),
  ]);

  return { session, shellContext };
}

export async function getAppShellContext() {
  const requestHeaders = await headers();
  return getAppShellContextFromCookie(requestHeaders.get("cookie"));
}

export async function getAccountSecuritySettings(): Promise<AccountSecuritySettingsFetchResult> {
  const requestHeaders = await headers();
  return getAccountSecuritySettingsFromCookie(requestHeaders.get("cookie"));
}

export async function getPersonalProfileSettings() {
  const requestHeaders = await headers();
  return getPersonalProfileSettingsFromCookie(requestHeaders.get("cookie"));
}

export async function getNotificationFilterSettings() {
  const requestHeaders = await headers();
  return getNotificationFilterSettingsFromCookie(requestHeaders.get("cookie"));
}

export async function getNotificationDeliverySettings() {
  const requestHeaders = await headers();
  return getNotificationDeliverySettingsFromCookie(
    requestHeaders.get("cookie"),
  );
}

export async function getPersonalAccessTokenList(): Promise<PersonalAccessTokenListFetchResult> {
  const requestHeaders = await headers();
  return getPersonalAccessTokenListFromCookie(requestHeaders.get("cookie"));
}

export async function getPersonalAccessTokenNewContext(): Promise<PersonalAccessTokenNewContextFetchResult> {
  const requestHeaders = await headers();
  return getPersonalAccessTokenNewContextFromCookie(
    requestHeaders.get("cookie"),
  );
}

export async function getKeySettings(): Promise<KeySettingsFetchResult> {
  const requestHeaders = await headers();
  return getKeySettingsFromCookie(requestHeaders.get("cookie"));
}

export async function getPublicUserProfile(
  username: string,
  options: { year?: number } = {},
) {
  const requestHeaders = await headers();
  return getPublicUserProfileFromCookie(
    requestHeaders.get("cookie"),
    username,
    options,
  );
}

export async function getPublicOrganizationProfile(org: string) {
  const requestHeaders = await headers();
  return getPublicOrganizationProfileFromCookie(
    requestHeaders.get("cookie"),
    org,
  );
}

export async function getOrganizationProfileSettings(
  org: string,
): Promise<OrganizationProfileSettingsFetchResult> {
  const requestHeaders = await headers();
  return getOrganizationProfileSettingsFromCookie(
    requestHeaders.get("cookie"),
    org,
  );
}

export async function getOrganizationMemberPrivileges(
  org: string,
): Promise<OrganizationMemberPrivilegesFetchResult> {
  const requestHeaders = await headers();
  return getOrganizationMemberPrivilegesFromCookie(
    requestHeaders.get("cookie"),
    org,
  );
}

export async function getOrganizationRepositories(
  org: string,
  query: OrganizationRepositoryListQuery = {},
) {
  const requestHeaders = await headers();
  return getOrganizationRepositoriesFromCookie(
    requestHeaders.get("cookie"),
    org,
    query,
  );
}

export async function getUserProjects(
  username: string,
  query: ProjectListQuery = {},
): Promise<ProjectListFetchResult> {
  const requestHeaders = await headers();
  return getUserProjectsFromCookie(
    requestHeaders.get("cookie"),
    username,
    query,
  );
}

export async function getOrganizationProjects(
  org: string,
  query: ProjectListQuery = {},
): Promise<ProjectListFetchResult> {
  const requestHeaders = await headers();
  return getOrganizationProjectsFromCookie(
    requestHeaders.get("cookie"),
    org,
    query,
  );
}

export async function getUserProjectWorkspace(
  username: string,
  projectNumber: number,
  query: ProjectWorkspaceQuery = {},
): Promise<ProjectWorkspaceFetchResult> {
  const requestHeaders = await headers();
  return getUserProjectWorkspaceFromCookie(
    requestHeaders.get("cookie"),
    username,
    projectNumber,
    query,
  );
}

export async function getOrganizationProjectWorkspace(
  org: string,
  projectNumber: number,
  query: ProjectWorkspaceQuery = {},
): Promise<ProjectWorkspaceFetchResult> {
  const requestHeaders = await headers();
  return getOrganizationProjectWorkspaceFromCookie(
    requestHeaders.get("cookie"),
    org,
    projectNumber,
    query,
  );
}

export async function getProjectItemDetail(
  projectId: string,
  itemId: string,
): Promise<ProjectItemDetailFetchResult> {
  const requestHeaders = await headers();
  return getProjectItemDetailFromCookie(
    requestHeaders.get("cookie"),
    projectId,
    itemId,
  );
}

export async function getUserProjectFieldSettings(
  username: string,
  projectNumber: number,
): Promise<ProjectFieldSettingsFetchResult> {
  const requestHeaders = await headers();
  return getUserProjectFieldSettingsFromCookie(
    requestHeaders.get("cookie"),
    username,
    projectNumber,
  );
}

export async function getOrganizationProjectFieldSettings(
  org: string,
  projectNumber: number,
): Promise<ProjectFieldSettingsFetchResult> {
  const requestHeaders = await headers();
  return getOrganizationProjectFieldSettingsFromCookie(
    requestHeaders.get("cookie"),
    org,
    projectNumber,
  );
}

export async function getRepositoryProjects(
  owner: string,
  repo: string,
  query: ProjectListQuery = {},
): Promise<ProjectListFetchResult> {
  const requestHeaders = await headers();
  return getRepositoryProjectsFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    query,
  );
}

export async function getOrganizationPeople(
  org: string,
  query: OrganizationPeopleListQuery = {},
) {
  const requestHeaders = await headers();
  return getOrganizationPeopleFromCookie(
    requestHeaders.get("cookie"),
    org,
    query,
  );
}

export async function getOrganizationPeopleAdmin(
  org: string,
  query: OrganizationPeopleAdminQuery = {},
) {
  const requestHeaders = await headers();
  return getOrganizationPeopleAdminFromCookie(
    requestHeaders.get("cookie"),
    org,
    query,
  );
}

export async function getOrganizationTeams(
  org: string,
  query: OrganizationTeamsQuery = {},
) {
  const requestHeaders = await headers();
  return getOrganizationTeamsFromCookie(
    requestHeaders.get("cookie"),
    org,
    query,
  );
}

export async function getOrganizationTeamDetail(org: string, teamSlug: string) {
  const requestHeaders = await headers();
  return getOrganizationTeamDetailFromCookie(
    requestHeaders.get("cookie"),
    org,
    teamSlug,
  );
}

export async function getUserPackages(
  username: string,
  query: OwnerPackageListQuery = {},
) {
  const requestHeaders = await headers();
  return getUserPackagesFromCookie(
    requestHeaders.get("cookie"),
    username,
    query,
  );
}

export async function getOrganizationPackages(
  org: string,
  query: OwnerPackageListQuery = {},
) {
  const requestHeaders = await headers();
  return getOrganizationPackagesFromCookie(
    requestHeaders.get("cookie"),
    org,
    query,
  );
}

export async function getUserPackageDetail(
  username: string,
  packageType: string,
  packageName: string,
  version?: string | null,
): Promise<PackageDetailFetchResult> {
  const requestHeaders = await headers();
  return getUserPackageDetailFromCookie(
    requestHeaders.get("cookie"),
    username,
    packageType,
    packageName,
    version,
  );
}

export async function getUserPackageSettings(
  username: string,
  packageType: string,
  packageName: string,
): Promise<PackageSettingsFetchResult> {
  const requestHeaders = await headers();
  return getUserPackageSettingsFromCookie(
    requestHeaders.get("cookie"),
    username,
    packageType,
    packageName,
  );
}

export async function getOrganizationPackageDetail(
  org: string,
  packageType: string,
  packageName: string,
  version?: string | null,
): Promise<PackageDetailFetchResult> {
  const requestHeaders = await headers();
  return getOrganizationPackageDetailFromCookie(
    requestHeaders.get("cookie"),
    org,
    packageType,
    packageName,
    version,
  );
}

export async function getOrganizationPackageSettings(
  org: string,
  packageType: string,
  packageName: string,
): Promise<PackageSettingsFetchResult> {
  const requestHeaders = await headers();
  return getOrganizationPackageSettingsFromCookie(
    requestHeaders.get("cookie"),
    org,
    packageType,
    packageName,
  );
}

export async function getProfileRepositories(
  username: string,
  query: ProfileRepositoryListQuery = {},
) {
  const requestHeaders = await headers();
  return getProfileRepositoriesFromCookie(
    requestHeaders.get("cookie"),
    username,
    query,
  );
}

export async function getProfileStars(
  username: string,
  query: ProfileRepositoryListQuery = {},
) {
  const requestHeaders = await headers();
  return getProfileStarsFromCookie(
    requestHeaders.get("cookie"),
    username,
    query,
  );
}

export async function searchGlobal(query: GlobalSearchQuery) {
  const requestHeaders = await headers();
  return searchGlobalFromCookie(requestHeaders.get("cookie"), query);
}

export async function searchCode(query: CodeSearchQuery) {
  const requestHeaders = await headers();
  return searchCodeFromCookie(requestHeaders.get("cookie"), query);
}

export async function searchCollaboration(query: CollaborationSearchQuery) {
  const requestHeaders = await headers();
  return searchCollaborationFromCookie(requestHeaders.get("cookie"), query);
}

export async function getSearchSuggestions(query: SearchSuggestionsQuery = {}) {
  const requestHeaders = await headers();
  return getSearchSuggestionsFromCookie(requestHeaders.get("cookie"), query);
}

export async function getDashboardSummary(query: DashboardSummaryQuery = {}) {
  const requestHeaders = await headers();
  return getDashboardSummaryFromCookie(requestHeaders.get("cookie"), query);
}

export async function getRepository(owner: string, repo: string) {
  const requestHeaders = await headers();
  return getRepositoryFromCookie(requestHeaders.get("cookie"), owner, repo);
}

export async function getRepositoryReleases(
  owner: string,
  repo: string,
  query: RepositoryReleaseListQuery = {},
) {
  const requestHeaders = await headers();
  return getRepositoryReleasesFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    query,
  );
}

export async function getRepositoryReleaseDetail(
  owner: string,
  repo: string,
  tag: string,
) {
  const requestHeaders = await headers();
  return getRepositoryReleaseDetailFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    tag,
  );
}

export async function getRepositoryReleaseTags(
  owner: string,
  repo: string,
  query: RepositoryReleaseListQuery = {},
) {
  const requestHeaders = await headers();
  return getRepositoryReleaseTagsFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    query,
  );
}

export async function getRepositoryReleaseManagementContext(
  owner: string,
  repo: string,
  releaseId?: string | null,
) {
  const requestHeaders = await headers();
  return getRepositoryReleaseManagementContextFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    releaseId,
  );
}

export async function getRepositorySettings(owner: string, repo: string) {
  const requestHeaders = await headers();
  return getRepositorySettingsFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
  );
}

export async function getRepositoryAccessSettings(owner: string, repo: string) {
  const requestHeaders = await headers();
  return getRepositoryAccessSettingsFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
  );
}

export async function getRepositoryBranchSettings(owner: string, repo: string) {
  const requestHeaders = await headers();
  return getRepositoryBranchSettingsFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
  );
}

export async function getRepositoryBranches(
  owner: string,
  repo: string,
  options: {
    tab?: string | null;
    query?: string | null;
    page?: number | null;
    pageSize?: number | null;
  } = {},
): Promise<RepositoryBranchesFetchResult> {
  const requestHeaders = await headers();
  return getRepositoryBranchesFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    options,
  );
}

export async function getRepositoryBranchActivity(
  owner: string,
  repo: string,
  branch: string,
): Promise<RepositoryBranchActivityFetchResult> {
  const requestHeaders = await headers();
  return getRepositoryBranchActivityFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    branch,
  );
}

export async function getRepositoryPulse(
  owner: string,
  repo: string,
  options: { period?: string | null } = {},
): Promise<RepositoryPulseFetchResult> {
  const requestHeaders = await headers();
  return getRepositoryPulseFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    options,
  );
}

export async function getRepositoryContributors(
  owner: string,
  repo: string,
  options: {
    period?: string | null;
    start?: string | null;
    end?: string | null;
  } = {},
): Promise<RepositoryContributorsFetchResult> {
  const requestHeaders = await headers();
  return getRepositoryContributorsFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    options,
  );
}

export async function getRepositoryTraffic(
  owner: string,
  repo: string,
): Promise<RepositoryTrafficFetchResult> {
  const requestHeaders = await headers();
  return getRepositoryTrafficFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
  );
}

export async function getRepositorySecurityOverview(
  owner: string,
  repo: string,
): Promise<RepositorySecurityOverviewFetchResult> {
  const requestHeaders = await headers();
  return getRepositorySecurityOverviewFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
  );
}

export async function getRepositorySecurityPolicy(
  owner: string,
  repo: string,
): Promise<RepositorySecurityPolicyFetchResult> {
  const requestHeaders = await headers();
  return getRepositorySecurityPolicyFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
  );
}

export async function getRepositoryDependabotAlerts(
  owner: string,
  repo: string,
  options: RepositoryDependabotAlertsQuery = {},
): Promise<RepositoryDependabotAlertsFetchResult> {
  const requestHeaders = await headers();
  return getRepositoryDependabotAlertsFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    options,
  );
}

export async function getRepositoryDependabotAlertDetail(
  owner: string,
  repo: string,
  alertId: string | number,
): Promise<RepositoryDependabotAlertDetailFetchResult> {
  const requestHeaders = await headers();
  return getRepositoryDependabotAlertDetailFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    alertId,
  );
}

export async function getRepositoryCodeScanningAlerts(
  owner: string,
  repo: string,
  options: RepositoryCodeScanningAlertsQuery = {},
): Promise<RepositoryCodeScanningAlertsFetchResult> {
  const requestHeaders = await headers();
  return getRepositoryCodeScanningAlertsFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    options,
  );
}

export async function getRepositoryCodeScanningAlertDetail(
  owner: string,
  repo: string,
  alertId: string | number,
): Promise<RepositoryCodeScanningAlertDetailFetchResult> {
  const requestHeaders = await headers();
  return getRepositoryCodeScanningAlertDetailFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    alertId,
  );
}

export async function getRepositorySecretScanningAlerts(
  owner: string,
  repo: string,
  options: RepositorySecretScanningAlertsQuery = {},
): Promise<RepositorySecretScanningAlertsFetchResult> {
  const requestHeaders = await headers();
  return getRepositorySecretScanningAlertsFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    options,
  );
}

export async function getRepositorySecretScanningAlertDetail(
  owner: string,
  repo: string,
  alertId: string | number,
): Promise<RepositorySecretScanningAlertDetailFetchResult> {
  const requestHeaders = await headers();
  return getRepositorySecretScanningAlertDetailFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    alertId,
  );
}

export async function getRepositorySecurityAdvisories(
  owner: string,
  repo: string,
  options: RepositorySecurityAdvisoriesQuery = {},
): Promise<RepositorySecurityAdvisoriesFetchResult> {
  const requestHeaders = await headers();
  return getRepositorySecurityAdvisoriesFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    options,
  );
}

export async function getRepositorySecurityAdvisoryDetail(
  owner: string,
  repo: string,
  ghsaId: string,
): Promise<RepositorySecurityAdvisoryDetailFetchResult> {
  const requestHeaders = await headers();
  return getRepositorySecurityAdvisoryDetailFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    ghsaId,
  );
}

export async function getRepositoryNetwork(
  owner: string,
  repo: string,
): Promise<RepositoryNetworkFetchResult> {
  const requestHeaders = await headers();
  return getRepositoryNetworkFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
  );
}

export async function getRepositoryForks(
  owner: string,
  repo: string,
  options: RepositoryForksQuery = {},
): Promise<RepositoryForksFetchResult> {
  const requestHeaders = await headers();
  return getRepositoryForksFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    options,
  );
}

export async function getRepositoryDependencies(
  owner: string,
  repo: string,
  options: RepositoryDependenciesQuery = {},
): Promise<RepositoryDependenciesFetchResult> {
  const requestHeaders = await headers();
  return getRepositoryDependenciesFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    options,
  );
}

export async function getRepositoryDependents(
  owner: string,
  repo: string,
  options: RepositoryDependentsQuery = {},
): Promise<RepositoryDependentsFetchResult> {
  const requestHeaders = await headers();
  return getRepositoryDependentsFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    options,
  );
}

export async function getRepositoryDiscussions(
  owner: string,
  repo: string,
  options: RepositoryDiscussionsQuery = {},
  categorySlug?: string | null,
) {
  const requestHeaders = await headers();
  return getRepositoryDiscussionsFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    options,
    categorySlug,
  );
}

export async function getRepositoryDiscussionCreation(
  owner: string,
  repo: string,
  options: RepositoryDiscussionCreationQuery = {},
) {
  const requestHeaders = await headers();
  return getRepositoryDiscussionCreationFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    options,
  );
}

export async function getRepositoryDiscussionDetail(
  owner: string,
  repo: string,
  discussionNumber: number,
  options: RepositoryDiscussionDetailQuery = {},
) {
  const requestHeaders = await headers();
  return getRepositoryDiscussionDetailFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    discussionNumber,
    options,
  );
}

export async function getRepositoryDiscussionCategorySettings(
  owner: string,
  repo: string,
) {
  const requestHeaders = await headers();
  return getRepositoryDiscussionCategorySettingsFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
  );
}

export async function getRepositoryDiscussionCategoryTemplate(
  owner: string,
  repo: string,
  categoryId: string,
) {
  const requestHeaders = await headers();
  return getRepositoryDiscussionCategoryTemplateFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    categoryId,
  );
}

export async function getRepositoryWebhookSettings(
  owner: string,
  repo: string,
) {
  const requestHeaders = await headers();
  return getRepositoryWebhookSettingsFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
  );
}

export async function getRepositoryActionsSecretsSettings(
  owner: string,
  repo: string,
) {
  const requestHeaders = await headers();
  return getRepositoryActionsSecretsSettingsFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
  );
}

export async function getRepositoryPagesSettings(owner: string, repo: string) {
  const requestHeaders = await headers();
  return getRepositoryPagesSettingsFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
  );
}

export async function getRepositoryWebhookDetail(
  owner: string,
  repo: string,
  hookId: string,
) {
  const requestHeaders = await headers();
  return getRepositoryWebhookDetailFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    hookId,
  );
}

export async function getRepositoryWebhookDeliveryDetail(
  owner: string,
  repo: string,
  hookId: string,
  deliveryId: string,
) {
  const requestHeaders = await headers();
  return getRepositoryWebhookDeliveryDetailFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    hookId,
    deliveryId,
  );
}

export async function getRepositoryIssueTemplates(owner: string, repo: string) {
  const requestHeaders = await headers();
  return getRepositoryIssueTemplatesFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
  );
}

export async function getRepositoryPath(
  owner: string,
  repo: string,
  refName: string,
  path: string,
  options: { page?: number; pageSize?: number } = {},
) {
  const requestHeaders = await headers();
  return getRepositoryPathFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    refName,
    path,
    options,
  );
}

export async function getRepositoryBlob(
  owner: string,
  repo: string,
  refName: string,
  path: string,
) {
  const requestHeaders = await headers();
  return getRepositoryBlobFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    refName,
    path,
  );
}

export async function getRepositoryBlame(
  owner: string,
  repo: string,
  refName: string,
  path: string,
) {
  const requestHeaders = await headers();
  return getRepositoryBlameFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    refName,
    path,
  );
}

export async function getRepositoryCommitHistory(
  owner: string,
  repo: string,
  refName: string,
  path: string,
  options: {
    author?: string | null;
    until?: string | null;
    page?: number | null;
    pageSize?: number | null;
  } = {},
) {
  const requestHeaders = await headers();
  return getRepositoryCommitHistoryFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    refName,
    path,
    options,
  );
}

export async function getRepositoryCommitDetail(
  owner: string,
  repo: string,
  sha: string,
) {
  const requestHeaders = await headers();
  return getRepositoryCommitDetailFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    sha,
  );
}

export async function getRepositoryRefs(owner: string, repo: string) {
  const requestHeaders = await headers();
  return getRepositoryRefsFromCookie(requestHeaders.get("cookie"), owner, repo);
}

export async function getRepositoryFileFinder(
  owner: string,
  repo: string,
  refName: string,
  query: string,
) {
  const requestHeaders = await headers();
  return getRepositoryFileFinderFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    refName,
    query,
  );
}

export async function getRepositoryCreationOptions() {
  const requestHeaders = await headers();
  return getRepositoryCreationOptionsFromCookie(requestHeaders.get("cookie"));
}

export async function getRepositoryImport(importId: string) {
  const requestHeaders = await headers();
  return getRepositoryImportFromCookie(requestHeaders.get("cookie"), importId);
}

export async function getRepositoryIssues(
  owner: string,
  repo: string,
  query: RepositoryIssueListQuery = {},
) {
  const requestHeaders = await headers();
  return getRepositoryIssuesFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    query,
  );
}

export async function getRepositoryPullRequests(
  owner: string,
  repo: string,
  query: RepositoryPullRequestListQuery = {},
) {
  const requestHeaders = await headers();
  return getRepositoryPullRequestsFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    query,
  );
}

export async function getRepositoryActionsDashboard(
  owner: string,
  repo: string,
  query: RepositoryActionsDashboardQuery = {},
) {
  const requestHeaders = await headers();
  return getRepositoryActionsDashboardFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    query,
  );
}

export async function getRepositoryActionsWorkflowDashboard(
  owner: string,
  repo: string,
  workflowFile: string,
  query: RepositoryActionsDashboardQuery = {},
) {
  const requestHeaders = await headers();
  return getRepositoryActionsWorkflowDashboardFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    workflowFile,
    query,
  );
}

export async function getRepositoryActionsRunDetail(
  owner: string,
  repo: string,
  runId: string,
) {
  const requestHeaders = await headers();
  return getRepositoryActionsRunDetailFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    runId,
  );
}

export async function getRepositoryActionsJobLogDetail(
  owner: string,
  repo: string,
  runId: string,
  jobId: string,
  query: {
    q?: string | null;
    selectedMatch?: number | null;
    timestamps?: boolean | null;
    raw?: boolean | null;
    page?: number | null;
    pageSize?: number | null;
  } = {},
) {
  const requestHeaders = await headers();
  return getRepositoryActionsJobLogDetailFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    runId,
    jobId,
    query,
  );
}

export async function getRepositoryPullRequest(
  owner: string,
  repo: string,
  number: number | string,
) {
  const requestHeaders = await headers();
  return getRepositoryPullRequestFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    number,
  );
}

export async function getRepositoryPullRequestTimeline(
  owner: string,
  repo: string,
  number: number | string,
) {
  const requestHeaders = await headers();
  return getRepositoryPullRequestTimelineFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    number,
  );
}

export async function getRepositoryPullRequestFiles(
  owner: string,
  repo: string,
  number: number | string,
  query: RepositoryPullRequestDiffQuery = {},
) {
  const requestHeaders = await headers();
  return getRepositoryPullRequestFilesFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    number,
    query,
  );
}

export async function getPullRequestCompare(
  owner: string,
  repo: string,
  base: string,
  head: string,
  options: {
    commits?: number;
    files?: number;
    headOwner?: string;
    headRepo?: string;
  } = {},
) {
  const requestHeaders = await headers();
  return getPullRequestCompareFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    base,
    head,
    options,
  );
}

export async function getRepositoryIssue(
  owner: string,
  repo: string,
  issueNumber: number | string,
) {
  const requestHeaders = await headers();
  return getRepositoryIssueFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    issueNumber,
  );
}

export async function getRepositoryIssueTimeline(
  owner: string,
  repo: string,
  issueNumber: number | string,
) {
  const requestHeaders = await headers();
  return getRepositoryIssueTimelineFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    issueNumber,
  );
}

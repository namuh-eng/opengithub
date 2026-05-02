import { headers } from "next/headers";
import {
  type CodeSearchQuery,
  type CollaborationSearchQuery,
  type DashboardSummaryQuery,
  type GlobalSearchQuery,
  getAppShellContextFromCookie,
  getDashboardSummaryFromCookie,
  getProfileRepositoriesFromCookie,
  getProfileStarsFromCookie,
  getPublicUserProfileFromCookie,
  getPullRequestCompareFromCookie,
  getRepositoryActionsDashboardFromCookie,
  getRepositoryActionsJobLogDetailFromCookie,
  getRepositoryActionsRunDetailFromCookie,
  getRepositoryActionsWorkflowDashboardFromCookie,
  getRepositoryBlameFromCookie,
  getRepositoryBlobFromCookie,
  getRepositoryCommitHistoryFromCookie,
  getRepositoryCreationOptionsFromCookie,
  getRepositoryFileFinderFromCookie,
  getRepositoryFromCookie,
  getRepositoryImportFromCookie,
  getRepositoryIssueFromCookie,
  getRepositoryIssuesFromCookie,
  getRepositoryIssueTemplatesFromCookie,
  getRepositoryIssueTimelineFromCookie,
  getRepositoryPathFromCookie,
  getRepositoryPullRequestFilesFromCookie,
  getRepositoryPullRequestFromCookie,
  getRepositoryPullRequestsFromCookie,
  getRepositoryPullRequestTimelineFromCookie,
  getRepositoryRefsFromCookie,
  getSearchSuggestionsFromCookie,
  getSessionFromHeaders,
  type ProfileRepositoryListQuery,
  type RepositoryActionsDashboardQuery,
  type RepositoryIssueListQuery,
  type RepositoryPullRequestDiffQuery,
  type RepositoryPullRequestListQuery,
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
) {
  const requestHeaders = await headers();
  return getRepositoryCommitHistoryFromCookie(
    requestHeaders.get("cookie"),
    owner,
    repo,
    refName,
    path,
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

import { AppShell } from "@/components/AppShell";
import { SearchResultsPage } from "@/components/SearchResultsPage";
import { activeSearchType } from "@/lib/navigation";
import {
  getSessionAndShellContext,
  searchCode,
  searchCollaboration,
  searchGlobal,
} from "@/lib/server-session";

type SearchPageProps = {
  searchParams?: Promise<Record<string, string | string[] | undefined>>;
};

function firstParam(value: string | string[] | undefined) {
  return Array.isArray(value) ? value[0] : value;
}

function qualifierToken(qualifier: string, value: string | undefined) {
  const trimmed = value?.trim();
  if (!trimmed) {
    return null;
  }
  const quoted = /\s/.test(trimmed)
    ? `"${trimmed.replaceAll('"', '\\"')}"`
    : trimmed;
  return `${qualifier}:${quoted}`;
}

function codeQueryFromParams(
  query: string,
  params: Record<string, string | string[] | undefined> | undefined,
) {
  const tokens = [query];
  const owner = qualifierToken("owner", firstParam(params?.owner));
  const symbol = qualifierToken("symbol", firstParam(params?.symbol));
  if (owner && !query.match(/(?:^|\s)(owner|org|user):/i)) {
    tokens.push(owner);
  }
  if (symbol && !query.match(/(?:^|\s)symbol:/i)) {
    tokens.push(symbol);
  }
  if (
    firstParam(params?.archived) === "false" &&
    !query.match(/(?:^|\s)(archived|is):/i)
  ) {
    tokens.push("archived:false");
  }
  return tokens.join(" ").trim();
}

export default async function SearchPage({ searchParams }: SearchPageProps) {
  const [{ session, shellContext }, params] = await Promise.all([
    getSessionAndShellContext(),
    searchParams,
  ]);
  const query = firstParam(params?.q)?.trim() ?? "";
  const activeType = activeSearchType(firstParam(params?.type));
  const codeQuery =
    activeType === "code" ? codeQueryFromParams(query, params) : query;
  const view =
    firstParam(params?.view) === "compact" ? "compact" : "comfortable";
  const saved = firstParam(params?.saved) === "1";
  const parsedPage = Number.parseInt(firstParam(params?.page) ?? "1", 10);
  const page = Number.isFinite(parsedPage) ? Math.max(1, parsedPage) : 1;
  const results =
    codeQuery.length > 0
      ? activeType === "code"
        ? await searchCode({
            query: codeQuery,
            page,
            pageSize: 30,
          })
        : activeType === "issues" || activeType === "pull_requests"
          ? await searchCollaboration({
              query,
              type: activeType,
              page,
              pageSize: 30,
              sort: firstParam(params?.sort),
            })
          : await searchGlobal({
              query,
              type: activeType,
              page,
              pageSize: 30,
            })
      : null;

  return (
    <AppShell session={session} shellContext={shellContext}>
      <SearchResultsPage
        activeType={activeType}
        query={codeQuery}
        saved={saved}
        results={results}
        view={view}
      />
    </AppShell>
  );
}

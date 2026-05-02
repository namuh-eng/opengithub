import { AppShell } from "@/components/AppShell";
import { SearchResultsPage } from "@/components/SearchResultsPage";
import { activeSearchType } from "@/lib/navigation";
import {
  getSessionAndShellContext,
  searchCode,
  searchGlobal,
} from "@/lib/server-session";

type SearchPageProps = {
  searchParams?: Promise<Record<string, string | string[] | undefined>>;
};

function firstParam(value: string | string[] | undefined) {
  return Array.isArray(value) ? value[0] : value;
}

export default async function SearchPage({ searchParams }: SearchPageProps) {
  const [{ session, shellContext }, params] = await Promise.all([
    getSessionAndShellContext(),
    searchParams,
  ]);
  const query = firstParam(params?.q)?.trim() ?? "";
  const activeType = activeSearchType(firstParam(params?.type));
  const parsedPage = Number.parseInt(firstParam(params?.page) ?? "1", 10);
  const page = Number.isFinite(parsedPage) ? Math.max(1, parsedPage) : 1;
  const results =
    query.length > 0
      ? activeType === "code"
        ? await searchCode({
            query,
            page,
            pageSize: 30,
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
        query={query}
        results={results}
      />
    </AppShell>
  );
}

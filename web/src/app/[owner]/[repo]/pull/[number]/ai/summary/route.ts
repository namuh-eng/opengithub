import { cookies } from "next/headers";
import { NextResponse } from "next/server";
import { regeneratePullRequestAiSummaryFromCookie } from "@/lib/api";

type Context = {
  params: Promise<{ owner: string; repo: string; number: string }>;
};

export async function POST(_request: Request, context: Context) {
  const { owner, repo, number } = await context.params;
  const cookie = (await cookies()).toString();
  const result = await regeneratePullRequestAiSummaryFromCookie(
    cookie,
    decodeURIComponent(owner),
    decodeURIComponent(repo),
    decodeURIComponent(number),
  );
  return NextResponse.json(result, {
    status: "error" in result ? result.status : 200,
  });
}

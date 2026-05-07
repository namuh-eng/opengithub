import { cookies } from "next/headers";
import { NextResponse } from "next/server";
import { regenerateRepositoryAiSummaryFromCookie } from "@/lib/api";

type Context = { params: Promise<{ owner: string; repo: string }> };

export async function POST(_request: Request, context: Context) {
  const { owner, repo } = await context.params;
  const cookie = (await cookies()).toString();
  const result = await regenerateRepositoryAiSummaryFromCookie(
    cookie,
    decodeURIComponent(owner),
    decodeURIComponent(repo),
  );
  return NextResponse.json(result, {
    status: "error" in result ? result.status : 200,
  });
}

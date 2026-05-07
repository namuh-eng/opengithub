import { cookies } from "next/headers";
import { NextResponse } from "next/server";
import { generateAiChangelogFromCookie } from "@/lib/api";

type Context = {
  params: Promise<{ owner: string; repo: string; tag: string }>;
};

export async function POST(_request: Request, context: Context) {
  const { owner, repo, tag } = await context.params;
  const cookie = (await cookies()).toString();
  const result = await generateAiChangelogFromCookie(
    cookie,
    decodeURIComponent(owner),
    decodeURIComponent(repo),
    { previousTag: null, targetTag: decodeURIComponent(tag) },
  );
  return NextResponse.json(result, {
    status: "error" in result ? result.status : 200,
  });
}

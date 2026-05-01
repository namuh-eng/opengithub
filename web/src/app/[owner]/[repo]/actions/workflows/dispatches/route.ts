import { apiBaseUrl } from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string }>;
};

export async function POST(request: Request, { params }: RouteContext) {
  const { owner, repo } = await params;
  const body = await request.json().catch(() => null);
  const workflowFile =
    typeof body === "object" &&
    body !== null &&
    "workflowFile" in body &&
    typeof body.workflowFile === "string"
      ? body.workflowFile
      : "";

  if (!workflowFile.trim()) {
    return Response.json(
      {
        error: {
          code: "validation_failed",
          message: "workflowFile is required.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(
      decodeURIComponent(owner),
    )}/${encodeURIComponent(
      decodeURIComponent(repo),
    )}/actions/workflows/${encodeURIComponent(workflowFile)}/dispatches`,
    {
      body: JSON.stringify({
        ref:
          typeof body === "object" && body !== null && "ref" in body
            ? body.ref
            : "",
        inputs:
          typeof body === "object" && body !== null && "inputs" in body
            ? body.inputs
            : {},
      }),
      headers: {
        "content-type": "application/json",
        ...(request.headers.get("cookie")
          ? { cookie: request.headers.get("cookie") as string }
          : {}),
      },
      method: "POST",
      cache: "no-store",
    },
  );

  return new Response(await response.text(), {
    status: response.status,
    headers: {
      "content-type":
        response.headers.get("content-type") ?? "application/json",
    },
  });
}

import Link from "next/link";
import { DeveloperCommandBlock } from "@/components/DeveloperCommandBlock";
import {
  type ApiDocMethod,
  apiEndpointDocs,
  errorEnvelopeExample,
  paginationExample,
} from "@/lib/api-docs";

const methodChipClass: Record<ApiDocMethod, string> = {
  GET: "chip info",
  POST: "chip ok",
  PATCH: "chip warn",
  DELETE: "chip err",
};

export function ApiDocsPage() {
  return (
    <article className="mx-auto max-w-6xl overflow-x-hidden px-4 py-8 sm:px-6">
      <div
        className="flex flex-col gap-6 pb-8 lg:flex-row lg:items-end lg:justify-between"
        style={{ borderBottom: "1px solid var(--line)" }}
      >
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            opengithub REST API
          </p>
          <h1 className="mt-2 t-h2">
            Build against implemented opengithub APIs
          </h1>
          <p
            className="mt-4 max-w-3xl t-body"
            style={{ color: "var(--ink-3)" }}
          >
            These endpoints are served by the Rust API and backed by
            opengithub-owned Postgres data. The catalog only lists APIs that are
            implemented in this build, with the same pagination and error
            envelopes used by the product UI.
          </p>
        </div>
        <div className="flex flex-wrap gap-3">
          <Link className="btn primary" href="/docs/git">
            Git docs
          </Link>
          <Link className="btn ghost" href="/settings/tokens">
            Tokens
          </Link>
          <Link className="btn ghost" href="/docs/get-started">
            Setup guide
          </Link>
        </div>
      </div>

      <section className="mt-8 grid gap-4 md:grid-cols-2">
        <div className="card p-4">
          <h2 className="t-h3">Authentication</h2>
          <p className="mt-2 t-body" style={{ color: "var(--ink-3)" }}>
            Browser clients use the signed opengithub session cookie created by
            Google OAuth. Personal access tokens are stored hashed and are
            reserved for Git, automation, and later token-management surfaces.
          </p>
        </div>
        <div className="card p-4">
          <h2 className="t-h3">Pagination and errors</h2>
          <p className="mt-2 t-body" style={{ color: "var(--ink-3)" }}>
            List endpoints accept page and pageSize and return
            items/total/page/pageSize. Failures use code/message envelopes with
            the matching HTTP status.
          </p>
        </div>
      </section>

      <section className="mt-8" aria-labelledby="endpoint-catalog-heading">
        <h2 id="endpoint-catalog-heading" className="t-h2">
          Endpoint catalog
        </h2>
        <div className="mt-4 space-y-4">
          {apiEndpointDocs.map((endpoint) => (
            <section
              key={endpoint.id}
              className="card p-4"
              id={endpoint.id}
              aria-labelledby={`${endpoint.id}-heading`}
            >
              <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                <div className="min-w-0">
                  <div className="flex flex-wrap items-center gap-2">
                    <span className={methodChipClass[endpoint.method]}>
                      {endpoint.method}
                    </span>
                    <code
                      className="t-mono break-all"
                      style={{ color: "var(--ink-1)" }}
                    >
                      {endpoint.path}
                    </code>
                  </div>
                  <h3 id={`${endpoint.id}-heading`} className="mt-3 t-h3">
                    {endpoint.title}
                  </h3>
                  <p className="mt-2 t-body" style={{ color: "var(--ink-3)" }}>
                    {endpoint.description}
                  </p>
                </div>
                <p
                  className="t-xs min-w-0 break-words rounded-md px-3 py-2 font-semibold"
                  style={{
                    border: "1px solid var(--line)",
                    background: "var(--surface-2)",
                    color: "var(--ink-3)",
                  }}
                >
                  {endpoint.auth}
                </p>
              </div>

              <details
                className="mt-4 rounded-md"
                style={{ border: "1px solid var(--line)" }}
              >
                <summary
                  className="cursor-pointer px-3 py-2 t-sm font-semibold hover:bg-[var(--hover)]"
                  style={{ color: "var(--accent)" }}
                >
                  Request and response examples
                </summary>
                <div
                  className="grid gap-4 p-3 lg:grid-cols-2"
                  style={{ borderTop: "1px solid var(--line)" }}
                >
                  {endpoint.request ? (
                    <DeveloperCommandBlock
                      copyLabel="Copy request"
                      label="Request"
                      value={endpoint.request}
                    />
                  ) : (
                    <DeveloperCommandBlock
                      copyLabel="Copy request"
                      label="Request"
                      value={`${endpoint.method} ${endpoint.path}`}
                    />
                  )}
                  <DeveloperCommandBlock
                    copyLabel="Copy response"
                    label="Response"
                    value={endpoint.response}
                  />
                </div>
              </details>

              <ul
                className="mt-3 list-inside list-disc space-y-1 t-sm leading-6"
                style={{ color: "var(--ink-3)" }}
              >
                {endpoint.notes.map((note) => (
                  <li key={note}>{note}</li>
                ))}
              </ul>
            </section>
          ))}
        </div>
      </section>

      <section className="mt-8 grid gap-4 lg:grid-cols-2">
        <div className="card p-4">
          <h2 className="t-h3">Pagination example</h2>
          <div className="mt-3">
            <DeveloperCommandBlock
              copyLabel="Copy pagination"
              label="List envelope"
              value={paginationExample}
            />
          </div>
        </div>
        <div className="card p-4">
          <h2 className="t-h3">Error example</h2>
          <div className="mt-3">
            <DeveloperCommandBlock
              copyLabel="Copy error"
              label="Error envelope"
              value={errorEnvelopeExample}
            />
          </div>
        </div>
      </section>
    </article>
  );
}

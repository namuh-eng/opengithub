import Link from "next/link";
import { googleStartUrl, sanitizeNextPath } from "@/lib/api";

const errorMessages: Record<string, string> = {
  oauth_failed: "Google sign-in could not be completed. Try again.",
};

type LoginPageProps = {
  searchParams: Promise<{
    error?: string | string[];
    next?: string | string[];
  }>;
};

function OpenGitHubMark() {
  return (
    <div
      aria-hidden="true"
      className="center"
      style={{
        width: 48,
        height: 48,
        borderRadius: "50%",
        background: "var(--ink-1)",
        color: "var(--bg)",
        boxShadow: "var(--shadow-sm)",
      }}
    >
      <svg aria-hidden="true" height="28" viewBox="0 0 32 32" width="28">
        <path
          d="M9.2 12.2c-.7-2.5-.2-4.6.8-6.2 1.9.2 3.5 1.2 4.7 2.4a12 12 0 0 1 2.6 0c1.2-1.2 2.8-2.2 4.7-2.4 1 1.6 1.5 3.7.8 6.2a8.3 8.3 0 0 1 1.3 4.7c0 5.6-3.4 8.3-8.1 8.3s-8.1-2.7-8.1-8.3c0-1.8.4-3.4 1.3-4.7Z"
          fill="currentColor"
        />
        <path
          d="M11.3 17.4c.5-.9 2.1-1.4 4.7-1.4s4.2.5 4.7 1.4c.5 1 .3 2.5-.5 3.4-.9 1-2.2 1.5-4.2 1.5s-3.3-.5-4.2-1.5c-.8-.9-1-2.4-.5-3.4Z"
          fill="var(--ink-1)"
          opacity=".88"
        />
        <path
          d="M13.2 18.7h.1M18.7 18.7h.1"
          stroke="currentColor"
          strokeLinecap="round"
          strokeWidth="1.7"
        />
      </svg>
    </div>
  );
}

export default async function LoginPage({ searchParams }: LoginPageProps) {
  const params = await searchParams;
  const nextPath = sanitizeNextPath(params.next);
  const errorCode = Array.isArray(params.error)
    ? params.error[0]
    : params.error;
  const errorMessage = errorCode ? errorMessages[errorCode] : null;

  return (
    <main
      className="page-enter center"
      style={{
        minHeight: "100vh",
        background: "var(--bg)",
        padding: "32px 18px",
      }}
    >
      <section
        aria-labelledby="login-heading"
        className="card"
        style={{
          width: "100%",
          maxWidth: 380,
          padding: "34px 30px 26px",
          textAlign: "center",
          background: "var(--surface)",
          boxShadow: "var(--shadow-md)",
        }}
      >
        <div style={{ display: "flex", justifyContent: "center" }}>
          <OpenGitHubMark />
        </div>

        <h1 className="t-h2" id="login-heading" style={{ marginTop: 20 }}>
          Sign in to opengithub
        </h1>
        <p className="t-sm" style={{ color: "var(--ink-3)", marginTop: 8 }}>
          Continue with your Google account. No passwords, no extra fields.
        </p>

        {errorMessage ? (
          <div
            className="card"
            role="alert"
            style={{
              marginTop: 20,
              padding: "12px 14px",
              fontSize: 13,
              background: "var(--err-soft)",
              borderColor: "transparent",
              color: "var(--err)",
              textAlign: "left",
            }}
          >
            {errorMessage}
          </div>
        ) : null}

        <a
          className="btn accent lg"
          href={googleStartUrl(nextPath)}
          style={{
            marginTop: 24,
            width: "100%",
            justifyContent: "center",
            height: 46,
            fontSize: 14,
          }}
        >
          Continue with Google
        </a>

        <p className="t-xs" style={{ marginTop: 22, color: "var(--ink-4)" }}>
          By signing in you agree to our{" "}
          <Link href="/terms" style={{ textDecoration: "underline" }}>
            terms
          </Link>{" "}
          and{" "}
          <Link href="/privacy" style={{ textDecoration: "underline" }}>
            privacy
          </Link>
          .
        </p>
      </section>
    </main>
  );
}

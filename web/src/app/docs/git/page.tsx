import Link from "next/link";
import { AppShell } from "@/components/AppShell";
import { DeveloperCommandBlock } from "@/components/DeveloperCommandBlock";
import { getSession } from "@/lib/server-session";

const publicRemote = "https://opengithub.namuh.co/mona/octo-app.git";
const rawUrl = "https://opengithub.namuh.co/mona/octo-app/raw/main/README.md";
const archiveUrl =
  "https://opengithub.namuh.co/mona/octo-app/archive/refs/heads/main.zip";
const cloneFetchCommands = `git clone ${publicRemote}
cd octo-app
git fetch origin main`;
const pushCommands = `git remote add origin ${publicRemote}
git branch -M main
git push -u origin main`;
const rawArchiveCommands = `curl -L ${rawUrl}
curl -L -o octo-app.zip ${archiveUrl}`;
const authenticatedGitCommands = `git clone https://opengithub.namuh.co/mona/private-app.git
# Username: your opengithub login
# Password: paste an oghp_ personal access token with repo:read

git -c credential.helper= \
  clone https://mona:oghp_your_token@opengithub.namuh.co/mona/private-app.git`;
const restPackageCommands = `curl -H "Authorization: Bearer oghp_your_token" \\
  https://opengithub.namuh.co/api/user

echo "$OPENGITHUB_TOKEN" | docker login opengithub.namuh.co \\
  -u "$OPENGITHUB_ACTOR" --password-stdin
docker pull opengithub.namuh.co/mona/octo-image:latest`;
const sshKeyCommands = `ssh-keygen -t ed25519 -C "mona@example.com"
cat ~/.ssh/id_ed25519.pub

# Add the public key in /settings/keys before SSH transport is enabled.
# opengithub stores the SHA256 fingerprint and never stores private keys.`;
const signingKeyCommands = `gpg --armor --export mona@example.com
git config --global user.signingkey 0F1E2D3C4B5A6978
git config --global commit.gpgsign true`;

export default async function GitDocsPage() {
  const session = await getSession();

  return (
    <AppShell session={session}>
      <article className="mx-auto max-w-4xl px-6 py-8">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          opengithub Git
        </p>
        <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
          Work with repositories over HTTPS
        </h1>
        <p className="t-body mt-4 max-w-3xl" style={{ color: "var(--ink-3)" }}>
          opengithub serves smart HTTP Git endpoints from its own Rust API.
          Public repositories can be cloned anonymously. Private repositories
          require a signed browser session or a personal access token with
          repository scope.
        </p>

        <section className="mt-8 space-y-3">
          <h2 className="t-h3" style={{ color: "var(--ink-1)" }}>
            Clone and fetch
          </h2>
          <DeveloperCommandBlock
            copyLabel="Copy clone"
            label="HTTPS commands"
            value={cloneFetchCommands}
          />
        </section>

        <section className="mt-8 space-y-3">
          <h2 className="t-h3" style={{ color: "var(--ink-1)" }}>
            Push changes
          </h2>
          <DeveloperCommandBlock
            copyLabel="Copy push"
            label="Push commands"
            value={pushCommands}
          />
          <p className="t-body" style={{ color: "var(--ink-3)" }}>
            For private repositories or command-line pushes, use a personal
            access token as the HTTPS password. Tokens are stored hashed by the
            API and are never returned after creation.
          </p>
        </section>

        <section className="mt-8 space-y-3">
          <h2 className="t-h3" style={{ color: "var(--ink-1)" }}>
            Authenticate with a token
          </h2>
          <p className="t-body" style={{ color: "var(--ink-3)" }}>
            Create fine-grained tokens for selected repositories whenever
            possible. Classic tokens remain available for older automation that
            expects broad legacy scopes. The token secret is shown once, then
            only the prefix, status, last-used time, and expiration remain
            visible in Developer Settings.
          </p>
          <DeveloperCommandBlock
            copyLabel="Copy authenticated clone"
            label="Token-backed Git"
            value={authenticatedGitCommands}
          />
          <ul
            className="list-inside list-disc space-y-1 t-sm leading-6"
            style={{ color: "var(--ink-3)" }}
          >
            <li>
              Use <span className="t-mono-sm">repo:read</span> for clone/fetch
              and <span className="t-mono-sm">repo:write</span> for push.
            </li>
            <li>
              Selected-repository fine-grained tokens work only for the
              repositories chosen during creation.
            </li>
            <li>
              Revoked or expired tokens fail immediately for Git, REST, and
              package registry requests.
            </li>
          </ul>
        </section>

        <section className="mt-8 space-y-3">
          <h2 className="t-h3" style={{ color: "var(--ink-1)" }}>
            REST and packages
          </h2>
          <p className="t-body" style={{ color: "var(--ink-3)" }}>
            The same personal access token can authorize REST API calls and OCI
            package pulls or pushes when its scopes include the matching
            permissions. Successful use refreshes the token last-used timestamp
            after the Rust API validates the hash, expiration, revocation state,
            scopes, and repository selection.
          </p>
          <DeveloperCommandBlock
            copyLabel="Copy automation auth"
            label="REST and registry"
            value={restPackageCommands}
          />
        </section>

        <section className="mt-8 space-y-3">
          <h2 className="t-h3" style={{ color: "var(--ink-1)" }}>
            SSH public keys
          </h2>
          <p className="t-body" style={{ color: "var(--ink-3)" }}>
            Add public SSH keys in Developer Settings so future SSH transport
            can validate the key fingerprint without storing private material.
            Revoked keys remain in account history, are excluded from
            authentication, and keep their audit trail.
          </p>
          <DeveloperCommandBlock
            copyLabel="Copy SSH setup"
            label="SSH key setup"
            value={sshKeyCommands}
          />
          <ul
            className="list-inside list-disc space-y-1 t-sm leading-6"
            style={{ color: "var(--ink-3)" }}
          >
            <li>
              Supported public-key rows show key type, SHA256 fingerprint,
              read/write access, source, added date, and last-used state.
            </li>
            <li>
              Duplicate active fingerprints are rejected before a new key is
              stored.
            </li>
          </ul>
        </section>

        <section className="mt-8 space-y-3">
          <h2 className="t-h3" style={{ color: "var(--ink-1)" }}>
            Commit signing and vigilant mode
          </h2>
          <p className="t-body" style={{ color: "var(--ink-3)" }}>
            Upload armored public GPG keys for commit and tag verification.
            Active GPG fingerprints mark matching user-attributed commits as
            verified. Vigilant mode flags unsigned or untrusted commits
            attributed to you as unverified until a trusted signing key matches.
          </p>
          <DeveloperCommandBlock
            copyLabel="Copy signing setup"
            label="GPG signing"
            value={signingKeyCommands}
          />
          <p className="t-sm" style={{ color: "var(--ink-3)" }}>
            Raw armored public keys are accepted only when adding a key. API
            responses and browser rows show fingerprints, key IDs, extracted
            email addresses, and revoked state without returning the armor.
          </p>
        </section>

        <section className="mt-8 space-y-3">
          <h2 className="t-h3" style={{ color: "var(--ink-1)" }}>
            Raw files and archives
          </h2>
          <DeveloperCommandBlock
            copyLabel="Copy curl"
            label="Raw and archive"
            value={rawArchiveCommands}
          />
        </section>

        <div className="mt-8 flex flex-wrap gap-3">
          <Link className="btn primary" href="/new">
            Create repository
          </Link>
          <Link className="btn ghost" href="/settings/tokens">
            Token settings
          </Link>
          <Link className="btn ghost" href="/settings/keys">
            Key settings
          </Link>
          <Link className="btn ghost" href="/docs/get-started">
            Setup guide
          </Link>
          <Link className="btn ghost" href="/docs/api">
            API docs
          </Link>
        </div>
      </article>
    </AppShell>
  );
}

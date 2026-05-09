"use client";

import Image from "next/image";
import Link from "next/link";
import { useEffect, useId, useRef, useState } from "react";
import { GlobalSearchModal } from "@/components/GlobalSearchModal";
import type { AppShellContext, AuthSession } from "@/lib/api";
import { CREATE_NAV_ITEMS, GLOBAL_NAV_ITEMS } from "@/lib/navigation";

type AppHeaderProps = {
  session: AuthSession;
  shellContext?: AppShellContext | null;
};

type MenuName = "global" | "create" | "avatar" | null;
type DrawerSectionLink = {
  href: string;
  label: string;
  detail?: string;
};

function userLabel(session: AuthSession) {
  return session.user?.display_name ?? session.user?.email ?? "Sign in";
}

function avatarText(session: AuthSession) {
  const label = userLabel(session).trim();
  return label.slice(0, 1).toUpperCase() || "O";
}

function Icon({ name }: { name: "menu" | "search" | "plus" | "bell" }) {
  if (name === "menu") {
    return (
      <svg aria-hidden="true" height="16" viewBox="0 0 16 16" width="16">
        <path
          d="M2.5 4h11M2.5 8h11M2.5 12h11"
          fill="none"
          stroke="currentColor"
          strokeLinecap="round"
          strokeWidth="1.5"
        />
      </svg>
    );
  }
  if (name === "search") {
    return (
      <svg aria-hidden="true" height="15" viewBox="0 0 16 16" width="15">
        <path
          d="m11.2 11.2 2.3 2.3M7.1 12.2a5.1 5.1 0 1 1 0-10.2 5.1 5.1 0 0 1 0 10.2Z"
          fill="none"
          stroke="currentColor"
          strokeLinecap="round"
          strokeWidth="1.5"
        />
      </svg>
    );
  }
  if (name === "plus") {
    return (
      <svg aria-hidden="true" height="16" viewBox="0 0 16 16" width="16">
        <path
          d="M8 3v10M3 8h10"
          fill="none"
          stroke="currentColor"
          strokeLinecap="round"
          strokeWidth="1.6"
        />
      </svg>
    );
  }
  return (
    <svg aria-hidden="true" height="16" viewBox="0 0 16 16" width="16">
      <path
        d="M4.5 6.8a3.5 3.5 0 1 1 7 0c0 3 1.3 3.7 1.3 3.7H3.2s1.3-.7 1.3-3.7ZM6.7 12.6h2.6"
        fill="none"
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth="1.4"
      />
    </svg>
  );
}

function MenuPanel({
  children,
  id,
  labelledBy,
}: {
  children: React.ReactNode;
  id: string;
  labelledBy: string;
}) {
  return (
    <div
      aria-labelledby={labelledBy}
      className="app-shell-menu absolute z-40 mt-2 w-72 rounded-md border py-2"
      id={id}
      role="menu"
    >
      {children}
    </div>
  );
}

function MenuLink({
  children,
  href,
}: {
  children: React.ReactNode;
  href: string;
}) {
  return (
    <Link
      className="app-shell-menu-link block px-4 py-2 t-sm"
      href={href}
      role="menuitem"
    >
      {children}
    </Link>
  );
}

function DrawerLink({
  detail,
  href,
  label,
  onNavigate,
}: DrawerSectionLink & { onNavigate: () => void }) {
  return (
    <Link
      className="app-shell-drawer-link flex min-h-11 items-center justify-between gap-3 border-b px-5 py-3"
      href={href}
      onClick={onNavigate}
    >
      <span className="min-w-0 truncate t-sm">{label}</span>
      {detail ? (
        <span className="shrink-0 t-xs" style={{ color: "var(--ink-3)" }}>
          {detail}
        </span>
      ) : null}
    </Link>
  );
}

function DrawerSection({
  empty,
  links,
  onNavigate,
  title,
}: {
  empty: string;
  links: DrawerSectionLink[];
  onNavigate: () => void;
  title: string;
}) {
  return (
    <section>
      <div className="px-5 pb-2 pt-5">
        <h2 className="t-label" style={{ color: "var(--ink-3)" }}>
          {title}
        </h2>
      </div>
      {links.length > 0 ? (
        links.map((link) => (
          <DrawerLink
            key={`${link.href}-${link.label}`}
            {...link}
            onNavigate={onNavigate}
          />
        ))
      ) : (
        <p className="px-5 py-3 t-xs" style={{ color: "var(--ink-3)" }}>
          {empty}
        </p>
      )}
    </section>
  );
}

export function AppHeader({ session, shellContext }: AppHeaderProps) {
  const signedIn = session.authenticated && session.user;
  const [openMenu, setOpenMenu] = useState<MenuName>(null);
  const [searchOpen, setSearchOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const headerRef = useRef<HTMLElement | null>(null);
  const mobileDrawerRef = useRef<HTMLDivElement | null>(null);
  const globalButtonId = useId();
  const createButtonId = useId();
  const avatarButtonId = useId();
  const globalMenuId = useId();
  const mobileGlobalMenuId = useId();
  const createMenuId = useId();
  const avatarMenuId = useId();
  const unreadCount = shellContext?.unreadNotificationCount ?? 0;
  const recentRepositories = shellContext?.recentRepositories ?? [];
  const organizations = shellContext?.organizations ?? [];
  const teams = shellContext?.teams ?? [];
  const quickLinks = shellContext?.quickLinks ?? [];
  const mobilePrimaryLinks =
    quickLinks.length > 0
      ? quickLinks.map((link) => ({
          href: link.href,
          label: link.label,
          detail: link.kind === "create" ? "Create" : undefined,
        }))
      : [
          ...GLOBAL_NAV_ITEMS.map((item) => ({
            href: item.href,
            label: item.label,
          })),
          ...CREATE_NAV_ITEMS.map((item) => ({
            href: item.href,
            label: item.label,
            detail: "Create",
          })),
        ];
  const mobileRepositoryLinks = recentRepositories.slice(0, 6).map((repo) => ({
    href: repo.href,
    label: `${repo.ownerLogin}/${repo.name}`,
    detail: repo.visibility,
  }));
  const mobileOrganizationLinks = [
    ...organizations.slice(0, 4).map((org) => ({
      href: org.href,
      label: org.displayName,
      detail: org.role,
    })),
    ...teams.slice(0, 4).map((team) => ({
      href: team.href,
      label: `${team.organizationSlug}/${team.name}`,
      detail: team.role,
    })),
  ];

  useEffect(() => {
    function onPointerDown(event: PointerEvent) {
      if (!headerRef.current?.contains(event.target as Node)) {
        setOpenMenu(null);
      }
    }

    function onKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        setOpenMenu(null);
      }
    }

    document.addEventListener("pointerdown", onPointerDown);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("pointerdown", onPointerDown);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, []);

  useEffect(() => {
    if (!signedIn) {
      return;
    }

    function onGlobalSearchShortcut(event: KeyboardEvent) {
      const target = event.target as HTMLElement | null;
      const tagName = target?.tagName?.toLowerCase();
      const isEditable =
        tagName === "input" ||
        tagName === "textarea" ||
        tagName === "select" ||
        target?.isContentEditable;
      const wantsSearch =
        event.key === "/" ||
        ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k");
      if (!wantsSearch || isEditable) {
        return;
      }
      event.preventDefault();
      setOpenMenu(null);
      setSearchOpen(true);
    }

    document.addEventListener("keydown", onGlobalSearchShortcut);
    return () =>
      document.removeEventListener("keydown", onGlobalSearchShortcut);
  }, [signedIn]);

  useEffect(() => {
    if (openMenu !== "global") {
      return;
    }

    const frame = window.requestAnimationFrame(() => {
      const firstFocusable = mobileDrawerRef.current?.querySelector<
        HTMLAnchorElement | HTMLButtonElement
      >("a, button");
      firstFocusable?.focus();
    });

    return () => window.cancelAnimationFrame(frame);
  }, [openMenu]);

  return (
    <header
      className="app-shell-header sticky top-0 z-30 border-b px-4"
      ref={headerRef}
    >
      <div className="mx-auto flex h-full max-w-[1240px] items-center gap-3">
        <div className="relative">
          <button
            aria-controls={globalMenuId}
            aria-expanded={openMenu === "global"}
            aria-label="Global menu"
            className="app-shell-icon-button btn ghost grid h-8 w-8 place-items-center p-0"
            id={globalButtonId}
            onClick={() => setOpenMenu(openMenu === "global" ? null : "global")}
            type="button"
          >
            <Icon name="menu" />
          </button>
          {openMenu === "global" ? (
            <>
              <div className="hidden md:block">
                <MenuPanel id={globalMenuId} labelledBy={globalButtonId}>
                  <div
                    className="border-b px-4 pb-2"
                    style={{ borderColor: "var(--line)" }}
                  >
                    <p className="t-label" style={{ color: "var(--ink-3)" }}>
                      Navigate
                    </p>
                  </div>
                  {GLOBAL_NAV_ITEMS.map((item) => (
                    <MenuLink href={item.href} key={item.href}>
                      {item.label}
                    </MenuLink>
                  ))}
                  <div
                    className="mt-2 border-t px-4 pb-1 pt-3"
                    style={{ borderColor: "var(--line)" }}
                  >
                    <p className="t-label" style={{ color: "var(--ink-3)" }}>
                      Recent repositories
                    </p>
                  </div>
                  {recentRepositories.length > 0 ? (
                    recentRepositories.slice(0, 5).map((repo) => (
                      <MenuLink href={repo.href} key={repo.id}>
                        <span className="t-mono-sm">
                          {repo.ownerLogin}/{repo.name}
                        </span>
                      </MenuLink>
                    ))
                  ) : (
                    <p className="px-4 py-2 t-xs">
                      No recent repositories yet.
                    </p>
                  )}
                  {organizations.length > 0 || teams.length > 0 ? (
                    <>
                      <div
                        className="mt-2 border-t px-4 pb-1 pt-3"
                        style={{ borderColor: "var(--line)" }}
                      >
                        <p
                          className="t-label"
                          style={{ color: "var(--ink-3)" }}
                        >
                          Organizations and teams
                        </p>
                      </div>
                      {organizations.slice(0, 3).map((org) => (
                        <MenuLink href={org.href} key={org.id}>
                          {org.displayName}
                        </MenuLink>
                      ))}
                      {teams.slice(0, 3).map((team) => (
                        <MenuLink href={team.href} key={team.id}>
                          {team.organizationSlug}/{team.name}
                        </MenuLink>
                      ))}
                    </>
                  ) : null}
                </MenuPanel>
              </div>
              <div
                aria-hidden="true"
                className="app-shell-drawer-backdrop fixed inset-0 z-40 md:hidden"
                onClick={() => setOpenMenu(null)}
              />
              <div
                aria-labelledby={globalButtonId}
                aria-modal="true"
                className="app-shell-drawer fixed bottom-0 left-0 top-0 z-50 flex w-[min(88vw,360px)] flex-col border-r md:hidden"
                id={mobileGlobalMenuId}
                ref={mobileDrawerRef}
                role="dialog"
              >
                <div
                  className="flex min-h-[var(--header-h)] items-center justify-between border-b px-5"
                  style={{ borderColor: "var(--line)" }}
                >
                  <div className="min-w-0">
                    <p className="t-label" style={{ color: "var(--ink-3)" }}>
                      Menu
                    </p>
                    <p className="truncate t-sm">{userLabel(session)}</p>
                  </div>
                  <button
                    aria-label="Close global menu"
                    className="app-shell-icon-button btn ghost grid h-8 w-8 place-items-center p-0"
                    onClick={() => setOpenMenu(null)}
                    type="button"
                  >
                    <Icon name="menu" />
                  </button>
                </div>
                <div className="min-h-0 flex-1 overflow-y-auto pb-6">
                  <DrawerSection
                    empty="No navigation links available."
                    links={mobilePrimaryLinks}
                    onNavigate={() => setOpenMenu(null)}
                    title="Navigate"
                  />
                  <DrawerSection
                    empty="No recent repositories yet."
                    links={mobileRepositoryLinks}
                    onNavigate={() => setOpenMenu(null)}
                    title="Recent repositories"
                  />
                  <DrawerSection
                    empty="No organizations or teams yet."
                    links={mobileOrganizationLinks}
                    onNavigate={() => setOpenMenu(null)}
                    title="Organizations and teams"
                  />
                </div>
              </div>
            </>
          ) : null}
        </div>

        <Link
          aria-label="opengithub dashboard"
          className="flex items-center gap-2"
          href={signedIn ? "/dashboard" : "/"}
        >
          <span
            className="grid h-8 w-8 place-items-center rounded-full t-h3"
            style={{
              background: "var(--accent)",
              color: "var(--surface)",
              fontFamily: "var(--display)",
            }}
          >
            o
          </span>
          <span
            className="hidden text-[18px] font-medium sm:inline"
            style={{ fontFamily: "var(--display)" }}
          >
            opengithub
          </span>
        </Link>

        {signedIn ? (
          <>
            <nav
              className="hidden items-center gap-1 md:flex"
              aria-label="Global"
            >
              {GLOBAL_NAV_ITEMS.slice(0, 3).map((item) => (
                <Link
                  className="rounded-md px-2.5 py-1.5 t-sm hover:opacity-75"
                  href={item.href}
                  key={item.href}
                >
                  {item.href === "/dashboard" ? "Home" : item.label}
                </Link>
              ))}
            </nav>

            {/* biome-ignore lint/a11y/useSemanticElements: React and jsdom do not recognize the native search element yet. */}
            <form
              action="/search"
              className="relative ml-auto hidden h-8 min-w-[220px] max-w-[360px] flex-1 items-center gap-2 rounded-md border px-2 lg:flex"
              onSubmit={() => setSearchOpen(false)}
              role="search"
              style={{
                background: "var(--surface)",
                borderColor: "var(--line-strong)",
                color: "var(--ink-3)",
              }}
            >
              <Icon name="search" />
              <input
                aria-label="Search or jump to"
                aria-controls={
                  searchOpen ? "global-search-suggestions" : undefined
                }
                autoComplete="off"
                className="min-w-0 flex-1 bg-transparent t-sm outline-none"
                name="q"
                onChange={(event) => {
                  setSearchQuery(event.target.value);
                  setSearchOpen(true);
                }}
                onFocus={() => setSearchOpen(true)}
                placeholder="Search or jump to..."
                style={{ color: "var(--ink-1)" }}
                type="search"
                value={searchQuery}
              />
              <input name="type" type="hidden" value="repositories" />
              <span className="kbd">/</span>
            </form>
            {searchOpen ? (
              <GlobalSearchModal
                initialQuery={searchQuery}
                onClose={() => setSearchOpen(false)}
              />
            ) : null}

            <div className="relative">
              <button
                aria-controls={createMenuId}
                aria-expanded={openMenu === "create"}
                aria-label="Create new"
                className="app-shell-icon-button btn ghost grid h-8 w-8 place-items-center p-0"
                id={createButtonId}
                onClick={() =>
                  setOpenMenu(openMenu === "create" ? null : "create")
                }
                type="button"
              >
                <Icon name="plus" />
              </button>
              {openMenu === "create" ? (
                <MenuPanel id={createMenuId} labelledBy={createButtonId}>
                  {CREATE_NAV_ITEMS.map((item) => (
                    <MenuLink href={item.href} key={item.href}>
                      {item.label}
                    </MenuLink>
                  ))}
                </MenuPanel>
              ) : null}
            </div>

            <Link
              aria-label={
                unreadCount > 0
                  ? `${unreadCount} unread notifications`
                  : "Notifications"
              }
              className="relative grid h-8 w-8 place-items-center rounded-md hover:opacity-75"
              href="/notifications"
            >
              <Icon name="bell" />
              {unreadCount > 0 ? (
                <span
                  className="absolute -right-1 -top-1 grid min-h-4 min-w-4 place-items-center rounded-full px-1 t-num"
                  style={{
                    background: "var(--accent)",
                    color: "var(--surface)",
                    fontSize: 10,
                  }}
                >
                  {unreadCount > 99 ? "99+" : unreadCount}
                </span>
              ) : null}
            </Link>

            <div className="relative">
              <button
                aria-controls={avatarMenuId}
                aria-expanded={openMenu === "avatar"}
                aria-label="Open user menu"
                className="grid h-8 w-8 place-items-center overflow-hidden rounded-full"
                id={avatarButtonId}
                onClick={() =>
                  setOpenMenu(openMenu === "avatar" ? null : "avatar")
                }
                type="button"
              >
                {session.user?.avatar_url ? (
                  <Image
                    alt=""
                    className="h-8 w-8 rounded-full"
                    height={32}
                    src={session.user.avatar_url}
                    width={32}
                  />
                ) : (
                  <span className="av sm">{avatarText(session)}</span>
                )}
              </button>
              {openMenu === "avatar" ? (
                <div className="absolute right-0">
                  <MenuPanel id={avatarMenuId} labelledBy={avatarButtonId}>
                    <div
                      className="border-b px-4 pb-2"
                      style={{ borderColor: "var(--line)" }}
                    >
                      <p className="t-xs">Signed in as</p>
                      <p className="truncate t-sm">{userLabel(session)}</p>
                    </div>
                    <MenuLink href="/settings/profile">Your profile</MenuLink>
                    <MenuLink href="/settings/tokens">
                      Developer settings
                    </MenuLink>
                    <MenuLink href="/logout">Sign out</MenuLink>
                  </MenuPanel>
                </div>
              ) : null}
            </div>
          </>
        ) : (
          <Link className="btn ml-auto" href="/login">
            Sign in
          </Link>
        )}
      </div>
    </header>
  );
}

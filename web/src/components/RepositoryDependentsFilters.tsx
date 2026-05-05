"use client";

import Link from "next/link";
import { useEffect, useId, useRef, useState } from "react";
import type { RepositoryDependentPackage } from "@/lib/api";
import { repositoryDependentsHref } from "@/lib/navigation";

type RepositoryDependentsFiltersProps = {
  owner: string;
  repo: string;
  ownerFilter: string | null;
  packageFilter: string | null;
  packages: RepositoryDependentPackage[];
};

function filterValue(value: string | null | undefined) {
  return value?.trim() || null;
}

function packageKey(item: RepositoryDependentPackage) {
  return `${item.package.ecosystem}:${item.package.name}`;
}

export function RepositoryDependentsFilters({
  owner,
  repo,
  ownerFilter,
  packageFilter,
  packages,
}: RepositoryDependentsFiltersProps) {
  const [draftOwner, setDraftOwner] = useState(ownerFilter ?? "");
  const [open, setOpen] = useState(false);
  const menuId = useId();
  const wrapperRef = useRef<HTMLDivElement>(null);
  const selectedPackage =
    packageFilter ??
    packages.find((item) => item.selected)?.package.name ??
    "All packages";

  useEffect(() => {
    if (!open) return;

    function closeOnOutsideClick(event: MouseEvent) {
      if (!wrapperRef.current?.contains(event.target as Node)) {
        setOpen(false);
      }
    }

    function closeOnEscape(event: KeyboardEvent) {
      if (event.key === "Escape") setOpen(false);
    }

    document.addEventListener("mousedown", closeOnOutsideClick);
    document.addEventListener("keydown", closeOnEscape);
    return () => {
      document.removeEventListener("mousedown", closeOnOutsideClick);
      document.removeEventListener("keydown", closeOnEscape);
    };
  }, [open]);

  const href = (
    next: Partial<{ package: string | null; owner: string | null }>,
  ) =>
    repositoryDependentsHref(owner, repo, {
      package:
        next.package !== undefined ? filterValue(next.package) : packageFilter,
      owner: next.owner !== undefined ? filterValue(next.owner) : ownerFilter,
    });

  return (
    <div className="grid gap-3">
      <div className="flex flex-wrap gap-2">
        <div className="relative" ref={wrapperRef}>
          <button
            aria-controls={open ? menuId : undefined}
            aria-expanded={open}
            aria-haspopup="menu"
            className="btn"
            onClick={() => setOpen((value) => !value)}
            type="button"
          >
            Package: {selectedPackage}
          </button>
          {open ? (
            <div
              aria-label="Package filter options"
              className="card absolute left-0 z-20 mt-2 grid min-w-72 gap-1 p-2"
              id={menuId}
              role="menu"
              style={{ background: "var(--surface)" }}
            >
              <Link
                aria-current={!packageFilter ? "page" : undefined}
                className="btn sm ghost justify-start"
                href={href({ package: null })}
                role="menuitem"
                style={{
                  background: !packageFilter ? "var(--surface-2)" : undefined,
                }}
              >
                All packages
                {!packageFilter ? (
                  <span className="chip active ml-auto">Selected</span>
                ) : null}
              </Link>
              {packages.map((item) => {
                const key = packageKey(item);
                const active =
                  packageFilter?.toLowerCase() === key.toLowerCase() ||
                  packageFilter?.toLowerCase() ===
                    item.package.name.toLowerCase();
                return (
                  <Link
                    aria-current={active ? "page" : undefined}
                    className="btn sm ghost justify-start"
                    href={href({ package: key })}
                    key={item.package.id}
                    role="menuitem"
                    style={{
                      background: active ? "var(--surface-2)" : undefined,
                    }}
                  >
                    <span className="break-words">
                      {item.package.ecosystem}:{item.package.name}
                    </span>
                    <span className="chip soft ml-auto">
                      {item.dependentCount}
                    </span>
                  </Link>
                );
              })}
            </div>
          ) : null}
        </div>
        <Link className="btn" href={repositoryDependentsHref(owner, repo)}>
          Clear filters
        </Link>
      </div>

      <form
        action={href({ owner: draftOwner })}
        className="flex flex-wrap gap-2"
      >
        <label className="input min-w-64 flex-1" htmlFor="dependent-owner">
          <span className="t-label" style={{ color: "var(--ink-3)" }}>
            Owner
          </span>
          <input
            id="dependent-owner"
            name="owner"
            onChange={(event) => setDraftOwner(event.target.value)}
            placeholder="Filter by owner"
            value={draftOwner}
          />
        </label>
        <button className="btn primary" type="submit">
          Apply owner
        </button>
      </form>

      <div className="flex flex-wrap gap-2">
        <span className="chip soft">Package {selectedPackage}</span>
        <span className="chip soft">
          Owner {filterValue(ownerFilter) ?? "all public owners"}
        </span>
      </div>
    </div>
  );
}

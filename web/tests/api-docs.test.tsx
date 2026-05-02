import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ApiDocsPage } from "@/components/ApiDocsPage";
import { apiEndpointDocs } from "@/lib/api-docs";

describe("ApiDocsPage", () => {
  it("documents every implemented api-001 resource family", () => {
    render(<ApiDocsPage />);

    expect(
      screen.getByRole("heading", {
        name: "Build against implemented opengithub APIs",
      }),
    ).toBeVisible();
    expect(apiEndpointDocs.length).toBeGreaterThanOrEqual(34);

    for (const endpoint of apiEndpointDocs) {
      const card = screen
        .getByRole("heading", { name: endpoint.title })
        .closest("section");
      expect(card).not.toBeNull();
      expect(
        within(card as HTMLElement).getByText(endpoint.method),
      ).toBeVisible();
      expect(
        within(card as HTMLElement).getByText(endpoint.path),
      ).toBeVisible();
      expect(
        within(card as HTMLElement).getByText(endpoint.auth),
      ).toBeVisible();
    }

    expect(screen.getByText("/api/user")).toBeVisible();
    expect(
      screen.getByText(
        "/api/orgs/{org}/repositories?q=router&type=public&language=Rust&page=1&pageSize=30",
      ),
    ).toBeVisible();
    expect(
      screen.getByText("/api/orgs/{org}/people?q=member&page=1&pageSize=30"),
    ).toBeVisible();
    expect(
      screen.getByText(/Private organizations return not_found/),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Signed-out and outside viewers see public members only/,
      ),
    ).toBeVisible();
    expect(screen.getByText("/api/repos/{owner}/{repo}/issues")).toBeVisible();
    expect(
      screen.getAllByText("/api/repos/{owner}/{repo}/settings")[0],
    ).toBeVisible();
    expect(
      screen.getByText(
        /Every successful write inserts a repository.settings.update audit event/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Archived repositories reject every settings mutation except unarchive/,
      ),
    ).toBeVisible();
    expect(screen.getByText("/api/repos/{owner}/{repo}/pulls")).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/pulls/{number}/files?view=unified&whitespace=show",
      ),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/pulls/{number}/reviews"),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/pulls/{number}/merge"),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/pulls/{number}.diff"),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/pulls/{number}.patch"),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/actions/dashboard?q=ci&status=success&page=1&pageSize=30",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/actions/workflows/{workflow_file}/dashboard?status=success",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/actions/workflows/{workflow_file}/dispatches",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/Invalid YAML keeps the workflow visible/),
    ).toBeVisible();
    expect(
      screen.getAllByText(
        "/api/repos/{owner}/{repo}/actions/workflows?page=1&pageSize=30",
      )[0],
    ).toBeVisible();
    expect(
      screen.getAllByText("/api/repos/{owner}/{repo}/actions/workflows")[0],
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/actions/workflows/{workflow_id}",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/actions/runs?page=1&pageSize=30",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/actions/workflows/{workflow_id}/runs",
      ),
    ).toBeVisible();
    expect(
      screen.getAllByText("/api/repos/{owner}/{repo}/actions/runs/{run_id}")[0],
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/actions/runs/{run_id}/detail",
      ),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/actions/runs/{run_id}/rerun"),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/actions/runs/{run_id}/cancel",
      ),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/actions/runs/{run_id}/logs"),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/actions/runs/{run_id}/jobs/{job_id}/detail?q=error&match=1&timestamps=true&raw=false",
      ),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/actions/log-preferences"),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/actions/jobs/{job_id}/logs/download",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/actions/runs/{run_id}/logs/archive",
      ),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/actions/recent-view"),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/packages"),
    ).toBeVisible();
    expect(
      screen.getByText("/api/search?q=router&type=code&page=1&pageSize=30"),
    ).toBeVisible();
    expect(
      screen.getByText("/api/search/suggestions?q=router&scope=all&limit=8"),
    ).toBeVisible();
    expect(screen.getByText("/api/search/saved-searches")).toBeVisible();
    expect(screen.getByText("/api/search/saved-searches/{id}")).toBeVisible();
    expect(
      screen.getByText(/Duplicate names for the same viewer/),
    ).toBeVisible();
  }, 10000);

  it("opens examples without placeholder links or inert controls", () => {
    const { container } = render(<ApiDocsPage />);

    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
    const linkHrefs = screen
      .getAllByRole("link")
      .map((link) => link.getAttribute("href"));
    expect(linkHrefs).toEqual(
      expect.arrayContaining([
        "/docs/git",
        "/docs/get-started",
        "/settings/tokens",
      ]),
    );
    for (const href of linkHrefs) {
      expect(href).toMatch(/^\/(?:docs|settings)\//);
    }

    const firstSummary = screen.getAllByText(
      "Request and response examples",
    )[0];
    const details = firstSummary.closest("details");
    expect(details).not.toBeNull();
    expect(details).not.toHaveAttribute("open");

    fireEvent.click(firstSummary);

    expect(details).toHaveAttribute("open");
    expect(
      screen.getByText((content) => content.includes('"login": "mona"')),
    ).toBeVisible();
    expect(
      screen.getByText((content) =>
        content.includes('"code": "validation_failed"'),
      ),
    ).toBeVisible();
    expect(
      screen.getByText((content) =>
        content.includes("GET /api/repos?page=2&pageSize=10"),
      ),
    ).toBeVisible();
  });

  it("copies request examples from docs snippets", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText },
    });

    render(<ApiDocsPage />);

    fireEvent.click(screen.getAllByText("Request and response examples")[0]);
    fireEvent.click(screen.getAllByRole("button", { name: "Copy request" })[0]);

    expect(writeText).toHaveBeenCalledWith("GET /api/user");
    expect(await screen.findByRole("status")).toHaveTextContent("Copied");
  });
});

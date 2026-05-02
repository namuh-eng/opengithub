import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { ProfileContributionGraph } from "@/components/ProfileContributionGraph";
import type { ProfileContributionSummary } from "@/lib/api";

function summary(
  overrides: Partial<ProfileContributionSummary> = {},
): ProfileContributionSummary {
  return {
    total: 9,
    year: 2026,
    days: [
      { date: "2026-01-01", count: 0, intensity: 0 },
      { date: "2026-01-02", count: 1, intensity: 1 },
      { date: "2026-02-14", count: 8, intensity: 4 },
    ],
    recentEvents: [],
    ...overrides,
  };
}

describe("ProfileContributionGraph", () => {
  it("renders accessible contribution cells, month labels, legend, and year links", () => {
    render(<ProfileContributionGraph login="ashley" summary={summary()} />);

    expect(
      screen.getByRole("heading", { name: "9 contributions in 2026" }),
    ).toBeVisible();
    expect(screen.getByText("Jan")).toBeVisible();
    expect(screen.getByText("Feb")).toBeVisible();
    expect(screen.getByText("Less")).toBeVisible();
    expect(screen.getByText("More")).toBeVisible();

    expect(
      screen.getByLabelText("No contributions on January 1, 2026"),
    ).toHaveAttribute("type", "button");
    expect(
      screen.getByLabelText("8 contributions on February 14, 2026"),
    ).toHaveAttribute("title", "8 contributions on February 14, 2026");

    expect(
      screen.getByRole("link", { name: "2026", current: "page" }),
    ).toHaveAttribute("href", "/ashley?year=2026");
    expect(screen.getByRole("link", { name: "2025" })).toHaveAttribute(
      "href",
      "/ashley?year=2025",
    );
  });

  it("renders an empty year state without dead controls", () => {
    const { container } = render(
      <ProfileContributionGraph
        login="long-user-name"
        summary={summary({ total: 0, days: [] })}
      />,
    );

    expect(
      screen.getByText("No public contributions are visible for 2026."),
    ).toBeVisible();
    expect(container.querySelector('a[href="#"], a:not([href])')).toBeNull();
  });
});

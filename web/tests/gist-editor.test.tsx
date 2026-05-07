import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { GistEditor } from "@/components/GistEditor";
import type { GistDetail } from "@/lib/api";

function gist(): GistDetail {
  return {
    id: "11111111-1111-4111-8111-111111111111",
    description: "Release helper",
    isPublic: false,
    owner: {
      id: "22222222-2222-4222-8222-222222222222",
      login: "mona",
      name: "Mona",
      avatarUrl: null,
      href: "/mona",
    },
    files: [
      {
        id: "33333333-3333-4333-8333-333333333333",
        filename: "release.ts",
        language: "TypeScript",
        sizeBytes: 28,
        contentSha: "abc123",
        content: "export const release = true;\n",
        position: 0,
      },
    ],
    commentsCount: 0,
    starsCount: 2,
    forksCount: 1,
    cloneUrl: "https://opengithub.namuh.co/gist/111.git",
    embedUrl: "https://opengithub.namuh.co/api/gists/111/embed.js",
    href: "/gist/11111111-1111-4111-8111-111111111111",
    createdAt: "2026-05-07T00:00:00Z",
    updatedAt: "2026-05-07T00:00:00Z",
    comments: [],
    viewer: { authenticated: true, canEdit: true, isStarred: false },
  };
}

describe("GistEditor", () => {
  it("adds multi-file payloads and preserves secret visibility", () => {
    render(<GistEditor action="/gist/actions" gist={gist()} />);

    expect(screen.getByDisplayValue("Release helper")).toBeVisible();
    expect(screen.getByLabelText("Gist file content")).toHaveValue(
      "export const release = true;\n",
    );
    expect(screen.getByRole("radio", { name: "Secret" })).toBeChecked();

    fireEvent.click(screen.getByRole("button", { name: "Add file" }));
    fireEvent.change(screen.getByDisplayValue("gistfile2.txt"), {
      target: { value: "notes.md" },
    });
    fireEvent.change(screen.getByLabelText("Gist file content"), {
      target: { value: "# Notes\n" },
    });

    const filesJson = document.querySelector<HTMLInputElement>(
      'input[name="filesJson"]',
    );
    expect(filesJson?.value).toContain("release.ts");
    expect(filesJson?.value).toContain("notes.md");
    expect(filesJson?.value).toContain("# Notes");
  });
});

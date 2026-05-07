"use client";

import { useState } from "react";
import type { GistDetail } from "@/lib/api";

type DraftFile = {
  key: string;
  filename: string;
  content: string;
};

type GistEditorProps = {
  action: string;
  gist?: GistDetail | null;
};

export function GistEditor({ action, gist }: GistEditorProps) {
  const [files, setFiles] = useState<DraftFile[]>(
    gist?.files.length
      ? gist.files.map((file) => ({
          key: file.id,
          filename: file.filename,
          content: file.content,
        }))
      : [
          {
            key: "initial-file",
            filename: "hello-opengithub.md",
            content: "# Hello opengithub\n",
          },
        ],
  );
  const [active, setActive] = useState(0);

  function updateFile(index: number, patch: Partial<DraftFile>) {
    setFiles((current) =>
      current.map((file, fileIndex) =>
        fileIndex === index ? { ...file, ...patch } : file,
      ),
    );
  }

  function addFile() {
    setFiles((current) => [
      ...current,
      {
        key: globalThis.crypto?.randomUUID?.() ?? `file-${Date.now()}`,
        filename: `gistfile${current.length + 1}.txt`,
        content: "",
      },
    ]);
    setActive(files.length);
  }

  function removeFile(index: number) {
    setFiles((current) =>
      current.length === 1
        ? current
        : current.filter((_, fileIndex) => fileIndex !== index),
    );
    setActive((current) => Math.max(0, Math.min(current, files.length - 2)));
  }

  const current = files[active] ?? files[0];

  return (
    <form action={action} className="grid gap-5" method="post">
      <input name="intent" type="hidden" value={gist ? "update" : "create"} />
      {gist ? <input name="gistId" type="hidden" value={gist.id} /> : null}
      <input name="filesJson" type="hidden" value={JSON.stringify(files)} />
      <section className="card grid gap-4 p-5">
        <label className="grid gap-2">
          <span className="t-label" style={{ color: "var(--ink-3)" }}>
            Description
          </span>
          <input
            className="input"
            defaultValue={gist?.description ?? ""}
            name="description"
            placeholder="A tiny script, config, note, or reproducible snippet"
          />
        </label>
        <fieldset className="flex flex-wrap gap-3">
          <legend className="t-label mb-2" style={{ color: "var(--ink-3)" }}>
            Visibility
          </legend>
          <label className="chip">
            <input
              defaultChecked={gist?.isPublic ?? true}
              name="visibility"
              type="radio"
              value="public"
            />{" "}
            Public
          </label>
          <label className="chip">
            <input
              defaultChecked={gist ? !gist.isPublic : false}
              name="visibility"
              type="radio"
              value="secret"
            />{" "}
            Secret
          </label>
        </fieldset>
      </section>

      <section className="card overflow-hidden">
        <div
          className="flex flex-wrap items-center justify-between gap-3 border-b px-4 py-3"
          style={{ borderColor: "var(--line)" }}
        >
          <div className="tabs" role="tablist">
            {files.map((file, index) => (
              <button
                className={`tab ${index === active ? "active" : ""}`}
                key={file.key}
                onClick={() => setActive(index)}
                type="button"
              >
                {file.filename || `File ${index + 1}`}
              </button>
            ))}
          </div>
          <button className="btn sm" onClick={addFile} type="button">
            Add file
          </button>
        </div>
        <div className="grid gap-4 p-4">
          <label className="grid gap-2">
            <span className="t-label" style={{ color: "var(--ink-3)" }}>
              Filename
            </span>
            <input
              className="input t-mono-sm"
              onChange={(event) =>
                updateFile(active, { filename: event.target.value })
              }
              value={current.filename}
            />
          </label>
          <textarea
            aria-label="Gist file content"
            className="input t-mono-sm"
            onChange={(event) =>
              updateFile(active, { content: event.target.value })
            }
            rows={18}
            style={{ resize: "vertical" }}
            value={current.content}
          />
          <div className="flex justify-between gap-3">
            <p className="t-xs" style={{ color: "var(--ink-3)" }}>
              Language is detected from the file extension after save.
            </p>
            <button
              className="btn sm"
              disabled={files.length === 1}
              onClick={() => removeFile(active)}
              type="button"
            >
              Remove file
            </button>
          </div>
        </div>
      </section>

      <div className="flex justify-end">
        <button className="btn accent" type="submit">
          {gist ? "Update gist" : "Create gist"}
        </button>
      </div>
    </form>
  );
}

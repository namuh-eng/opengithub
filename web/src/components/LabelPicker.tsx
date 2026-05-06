"use client";

import { useMemo, useState } from "react";
import type { IssueListLabel } from "@/lib/api";

type LabelPickerProps = {
  disabled?: boolean;
  labels: IssueListLabel[];
  selectedLabels: IssueListLabel[];
  title: string;
  onCancel: () => void;
  onSave: (labels: IssueListLabel[]) => void;
};

function selectedSet(labels: IssueListLabel[]) {
  return new Set(labels.map((label) => label.id));
}

export function LabelPicker({
  disabled = false,
  labels,
  selectedLabels,
  title,
  onCancel,
  onSave,
}: LabelPickerProps) {
  const [query, setQuery] = useState("");
  const [selectedIds, setSelectedIds] = useState(() =>
    selectedSet(selectedLabels),
  );
  const filteredLabels = useMemo(() => {
    const needle = query.trim().toLowerCase();
    if (!needle) {
      return labels;
    }
    return labels.filter((label) => {
      const description = label.description ?? "";
      return (
        label.name.toLowerCase().includes(needle) ||
        description.toLowerCase().includes(needle)
      );
    });
  }, [labels, query]);
  const selectedCount = selectedIds.size;

  function toggle(label: IssueListLabel) {
    setSelectedIds((current) => {
      const next = new Set(current);
      if (next.has(label.id)) {
        next.delete(label.id);
      } else {
        next.add(label.id);
      }
      return next;
    });
  }

  function save() {
    onSave(labels.filter((label) => selectedIds.has(label.id)));
  }

  return (
    <div aria-label={title} className="card mb-3 overflow-hidden" role="dialog">
      <div className="border-b p-3" style={{ borderColor: "var(--line)" }}>
        <label className="t-label" htmlFor={`${title}-search`}>
          Search labels
        </label>
        <input
          className="input mt-2 h-9 w-full px-3 t-sm"
          disabled={disabled}
          id={`${title}-search`}
          onChange={(event) => setQuery(event.currentTarget.value)}
          placeholder="Filter by name or description"
          value={query}
        />
      </div>
      <div className="max-h-72 overflow-y-auto p-2">
        {filteredLabels.length ? (
          filteredLabels.map((label) => {
            const selected = selectedIds.has(label.id);
            return (
              <label
                className="flex cursor-pointer items-start gap-3 rounded-[var(--radius)] px-2 py-2 t-sm hover:bg-[var(--surface-2)]"
                key={label.id}
              >
                <input
                  checked={selected}
                  className="mt-1"
                  disabled={disabled}
                  onChange={() => toggle(label)}
                  type="checkbox"
                />
                <span
                  aria-hidden="true"
                  className="mt-1 inline-block h-2.5 w-2.5 shrink-0 rounded-full"
                  style={{ background: label.color }}
                />
                <span className="min-w-0 flex-1">
                  <span className="block break-words font-medium">
                    {label.name}
                  </span>
                  {label.description ? (
                    <span className="t-xs block break-words">
                      {label.description}
                    </span>
                  ) : null}
                </span>
              </label>
            );
          })
        ) : (
          <p className="p-2 t-xs">No labels match this search.</p>
        )}
      </div>
      <div
        className="flex flex-wrap items-center justify-between gap-2 border-t p-3"
        style={{ borderColor: "var(--line)" }}
      >
        <span className="t-xs">
          <span className="t-num">{selectedCount}</span> selected
        </span>
        <div className="flex gap-2">
          <button
            className="btn sm"
            disabled={disabled}
            onClick={onCancel}
            type="button"
          >
            Cancel
          </button>
          <button
            className="btn accent sm"
            disabled={disabled}
            onClick={save}
            type="button"
          >
            Save labels
          </button>
        </div>
      </div>
    </div>
  );
}

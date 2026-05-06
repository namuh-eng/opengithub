"use client";

import { useMemo, useState } from "react";
import type { IssueListMilestone } from "@/lib/api";

type MilestonePickerProps = {
  disabled?: boolean;
  milestones: IssueListMilestone[];
  onCancel: () => void;
  onSave: (milestone: IssueListMilestone | null) => void;
  selectedMilestone: IssueListMilestone | null;
  title: string;
};

export function MilestonePicker({
  disabled = false,
  milestones,
  onCancel,
  onSave,
  selectedMilestone,
  title,
}: MilestonePickerProps) {
  const [query, setQuery] = useState("");
  const filtered = useMemo(() => {
    const needle = query.trim().toLowerCase();
    if (!needle) return milestones;
    return milestones.filter((milestone) =>
      milestone.title.toLowerCase().includes(needle),
    );
  }, [milestones, query]);

  return (
    <div aria-label={title} className="card mb-3 p-3" role="dialog">
      <label className="grid gap-2 t-sm">
        <span className="t-label">Filter milestones</span>
        <input
          className="input"
          onChange={(event) => setQuery(event.target.value)}
          placeholder="Search milestones"
          value={query}
        />
      </label>
      <div className="mt-3 grid gap-2" role="listbox">
        <button
          aria-pressed={selectedMilestone === null}
          className="btn ghost sm w-full justify-start"
          disabled={disabled}
          onClick={() => onSave(null)}
          type="button"
        >
          No milestone
        </button>
        {filtered.length ? (
          filtered.map((milestone) => (
            <button
              aria-pressed={selectedMilestone?.id === milestone.id}
              className="btn ghost sm w-full justify-start"
              disabled={disabled}
              key={milestone.id}
              onClick={() => onSave(milestone)}
              type="button"
            >
              <span className="min-w-0 truncate">{milestone.title}</span>
              <span className="chip soft ml-auto">{milestone.state}</span>
            </button>
          ))
        ) : (
          <p className="t-xs px-2 py-3">No milestones match this search.</p>
        )}
      </div>
      <div className="mt-3 flex justify-end">
        <button className="btn sm" onClick={onCancel} type="button">
          Cancel
        </button>
      </div>
    </div>
  );
}

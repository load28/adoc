import { createTwoFilesPatch } from "diff";

/** Unified diff between two versions of a document (DESIGN.md §11). */
export function unifiedDiff(path: string, before: string, after: string): string {
  return createTwoFilesPatch(path, path, before, after, "current", "proposed", {
    context: 3,
  });
}

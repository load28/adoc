export const workspaceRealtimeEvents = [
  "DOCUMENT_CHANGED",
  "VERSION_PUBLISHED",
  "DISCUSSION_CHANGED",
  "MESSAGE_CHANGED",
  "REVIEW_CHANGED",
  "REFERENCE_CHANGED",
  "VOCABULARY_CHANGED",
  "INBOX_CHANGED",
] as const;

export type WorkspaceRealtimeEvent = (typeof workspaceRealtimeEvents)[number];

export function invalidationRoots(event: WorkspaceRealtimeEvent): readonly string[] {
  switch (event) {
    case "DOCUMENT_CHANGED":
      return ["document", "discussion", "backlinks", "search"];
    case "VERSION_PUBLISHED":
      return ["document", "versions", "search"];
    case "DISCUSSION_CHANGED":
    case "MESSAGE_CHANGED":
      return ["discussion", "discussion-detail", "inbox"];
    case "REVIEW_CHANGED":
      return ["review", "inbox", "document"];
    case "REFERENCE_CHANGED":
      return ["backlinks", "discussion-detail", "search"];
    case "VOCABULARY_CHANGED":
      return ["vocabulary", "search"];
    case "INBOX_CHANGED":
      return ["inbox"];
  }
}

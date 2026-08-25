const documentModes = ["published", "draft"] as const;
const documentPanels = ["discussion", "review", "history", "references", "ai"] as const;
const settingsSections = ["members", "groups", "permissions", "writing", "ai", "audit"] as const;

type DocumentMode = (typeof documentModes)[number];
type DocumentPanel = (typeof documentPanels)[number];
export type SettingsSection = (typeof settingsSections)[number];

export type DocumentSearch = {
  mode: DocumentMode;
  panel?: DocumentPanel;
  discussion?: string;
  review?: string;
  job?: string;
  proposal?: string;
  from?: string;
  to?: string;
  region?: string;
};

function optionalBounded(value: unknown): string | undefined {
  if (typeof value !== "string") return undefined;
  const normalized = value.trim();
  return normalized.length > 0 && normalized.length <= 200 ? normalized : undefined;
}

export function parseDocumentSearch(input: Record<string, unknown>): DocumentSearch {
  const mode = documentModes.includes(input.mode as DocumentMode)
    ? (input.mode as DocumentMode)
    : "published";
  const panel = documentPanels.includes(input.panel as DocumentPanel)
    ? (input.panel as DocumentPanel)
    : undefined;
  return {
    mode,
    ...(panel ? { panel } : {}),
    ...(optionalBounded(input.discussion) ? { discussion: optionalBounded(input.discussion) } : {}),
    ...(optionalBounded(input.review) ? { review: optionalBounded(input.review) } : {}),
    ...(optionalBounded(input.job) ? { job: optionalBounded(input.job) } : {}),
    ...(optionalBounded(input.proposal) ? { proposal: optionalBounded(input.proposal) } : {}),
    ...(optionalBounded(input.from) ? { from: optionalBounded(input.from) } : {}),
    ...(optionalBounded(input.to) ? { to: optionalBounded(input.to) } : {}),
    ...(optionalBounded(input.region) ? { region: optionalBounded(input.region) } : {}),
  };
}

export function parseSettingsSection(value: unknown): SettingsSection | undefined {
  return settingsSections.includes(value as SettingsSection)
    ? (value as SettingsSection)
    : undefined;
}

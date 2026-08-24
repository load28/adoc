import { parse as parseYaml, stringify as stringifyYaml } from "yaml";

export interface FrontmatterSplit {
  /** Parsed YAML data (empty object when no frontmatter). */
  data: Record<string, unknown>;
  /** Markdown body after the frontmatter block. */
  body: string;
}

const FM_OPEN = /^---\r?\n/;

/** Split a Markdown source into YAML frontmatter and body. */
export function splitFrontmatter(source: string): FrontmatterSplit {
  if (!FM_OPEN.test(source)) return { data: {}, body: source };

  const lines = source.split(/\r?\n/);
  // lines[0] === '---'; find the closing fence.
  let close = -1;
  for (let i = 1; i < lines.length; i++) {
    if (lines[i].trim() === "---") {
      close = i;
      break;
    }
  }
  if (close === -1) return { data: {}, body: source };

  const yamlText = lines.slice(1, close).join("\n");
  const body = lines.slice(close + 1).join("\n").replace(/^\r?\n/, "");
  const parsed = yamlText.trim() === "" ? {} : parseYaml(yamlText);
  if (parsed === null || typeof parsed !== "object" || Array.isArray(parsed)) {
    return { data: {}, body: source };
  }
  return { data: parsed as Record<string, unknown>, body };
}

/** Render frontmatter + body back to a Markdown source string. */
export function joinFrontmatter(data: Record<string, unknown>, body: string): string {
  const yamlText = stringifyYaml(data).trimEnd();
  return `---\n${yamlText}\n---\n\n${body.replace(/^\n+/, "")}`;
}

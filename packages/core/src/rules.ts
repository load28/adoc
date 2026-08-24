import type { WritingRule } from "./types.js";

/**
 * Cognitive Writing Rules (DESIGN.md §8).
 *
 * These ship with the app. Team rules from `.teamdoc/writing-rules.yaml`
 * are merged on top; teams can also disable individual cognitive rules.
 */
export const COGNITIVE_RULES: WritingRule[] = [
  {
    id: "C001",
    name: "Conclusion First",
    description: "핵심 결론이나 문서의 목적을 가능한 초반에 제공한다.",
    origin: "cognitive",
  },
  {
    id: "C002",
    name: "One Idea Per Paragraph",
    description: "하나의 문단에 서로 다른 여러 주장을 섞지 않는다.",
    origin: "cognitive",
  },
  {
    id: "C003",
    name: "Claim Near Evidence",
    description: "주장과 그 근거를 가능한 가까이 배치한다.",
    origin: "cognitive",
  },
  {
    id: "C004",
    name: "Progressive Disclosure",
    description: "독자가 아직 필요하지 않은 세부 구현을 상위 개념보다 먼저 설명하지 않는다.",
    origin: "cognitive",
  },
  {
    id: "C005",
    name: "Remove Redundancy",
    description: "같은 의미를 다른 표현으로 반복하지 않는다.",
    origin: "cognitive",
  },
  {
    id: "C006",
    name: "Decision Requires Rationale",
    description: "Decision에는 반드시 결정 이유가 존재해야 한다.",
    appliesTo: ["decision"],
    origin: "cognitive",
  },
  {
    id: "C007",
    name: "Alternatives Require Rejection Reason",
    description: "검토한 대안에는 선택하지 않은 이유가 있어야 한다.",
    appliesTo: ["alternatives"],
    origin: "cognitive",
  },
  {
    id: "C008",
    name: "Meaningful Grouping",
    description: "긴 나열은 의미 단위로 그룹화한다.",
    origin: "cognitive",
  },
  {
    id: "C009",
    name: "Contextual Terminology",
    description: "전문 용어 설명은 독자의 이해에 필요한 경우에만 제공한다.",
    origin: "cognitive",
  },
  {
    id: "C010",
    name: "Problem Before Detail",
    description: "구현 세부사항보다 해결하려는 문제를 먼저 이해시킨다.",
    origin: "cognitive",
  },
];

export interface WritingRulesConfig {
  /** Team-defined rules added on top of the cognitive rules. */
  rules?: { id: string; name?: string; description: string; appliesTo?: string[] }[];
  /** Cognitive rule ids the team chose to disable. */
  disable?: string[];
}

/** Merge team rules (from parsed writing-rules.yaml data) over the defaults. */
export function mergeWritingRules(config: WritingRulesConfig | undefined): WritingRule[] {
  const disabled = new Set(config?.disable ?? []);
  const merged: WritingRule[] = COGNITIVE_RULES.filter((r) => !disabled.has(r.id));
  for (const raw of config?.rules ?? []) {
    merged.push({
      id: raw.id,
      name: raw.name ?? raw.id,
      description: raw.description,
      appliesTo: raw.appliesTo,
      origin: "team",
    });
  }
  return merged;
}

/** Select the rules relevant to a set of section roles (undefined = all). */
export function rulesForRoles(rules: WritingRule[], roles?: string[]): WritingRule[] {
  if (!roles || roles.length === 0) return rules;
  const roleSet = new Set(roles);
  return rules.filter((r) => !r.appliesTo || r.appliesTo.some((role) => roleSet.has(role)));
}

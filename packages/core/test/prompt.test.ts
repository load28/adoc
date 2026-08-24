import { describe, expect, it } from "vitest";
import {
  DESIGN_TYPE,
  COGNITIVE_RULES,
  compilePrompt,
  compileIntentPrompt,
  mergeWritingRules,
  rulesForRoles,
  type WritingRequest,
} from "@adoc/core";

describe("Prompt Compiler (§13)", () => {
  const base: WritingRequest = {
    task: "compose",
    documentType: DESIGN_TYPE,
    source: [{ kind: "notes", label: "Raw notes", text: "서버 세션으로 바꾸자" }],
    context: [{ kind: "decision", label: "Decision #12", text: "IR을 도입했다" }],
    rules: COGNITIVE_RULES,
  };

  it("embeds structure, rules, source and context into the IR", () => {
    const ir = compilePrompt(base);
    expect(ir.output).toBe("markdown-document");
    expect(ir.system).toContain('"## Summary" (role: summary, required)');
    expect(ir.system).toContain("[C001] Conclusion First");
    expect(ir.user).toContain("서버 세션으로 바꾸자");
    expect(ir.user).toContain("Decision #12");
    expect(ir.user).toContain("Return ONLY the complete Markdown document body");
  });

  it("selects the right output contract per task", () => {
    expect(compilePrompt({ ...base, task: "critique" }).output).toBe("json-suggestions");
    expect(compilePrompt({ ...base, task: "merge" }).output).toBe("json-merge");
    expect(compileIntentPrompt("memo").output).toBe("json-intent");
  });

  it("restricts rewrite to target sections", () => {
    const ir = compilePrompt({ ...base, task: "rewrite", target: ["decision"], goal: "근거 강화" });
    expect(ir.user).toContain("rewrite ONLY these sections: decision");
    expect(ir.user).toContain("Rewrite goal: 근거 강화");
  });
});

describe("Writing rules (§8)", () => {
  it("merges team rules and honors disable", () => {
    const rules = mergeWritingRules({
      rules: [{ id: "T001", description: "본문은 한국어로 작성한다." }],
      disable: ["C009"],
    });
    expect(rules.find((r) => r.id === "T001")?.origin).toBe("team");
    expect(rules.find((r) => r.id === "C009")).toBeUndefined();
    expect(rules.find((r) => r.id === "C001")).toBeDefined();
  });

  it("filters rules by section role", () => {
    const filtered = rulesForRoles(COGNITIVE_RULES, ["decision"]);
    expect(filtered.some((r) => r.id === "C006")).toBe(true);
    expect(filtered.some((r) => r.id === "C007")).toBe(false); // alternatives only
    expect(filtered.some((r) => r.id === "C001")).toBe(true); // document-wide
  });
});

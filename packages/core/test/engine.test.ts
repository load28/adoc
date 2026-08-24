import { describe, expect, it } from "vitest";
import {
  DocumentTypeRegistry,
  MockAgentAdapter,
  WritingEngine,
  COGNITIVE_RULES,
  parseDocument,
  type AgentRequest,
} from "@adoc/core";

const registry = new DocumentTypeRegistry();

/** Mock agent that answers by prompt output contract, like a real agent would. */
function makeAdapter(): MockAgentAdapter {
  return new MockAgentAdapter((request: AgentRequest) => {
    switch (request.prompt.output) {
      case "json-intent":
        return JSON.stringify({
          intent: "DESIGN_PROPOSAL",
          problem: "Client-side token lifecycle management is duplicated.",
          motivation: ["duplicated refresh logic", "mobile complexity"],
          proposal: "Move authentication state to server sessions.",
          tradeoffs: ["Increased server dependency."],
          evidence: ["GitHub PR #124"],
          unknowns: ["migration strategy"],
        });
      case "markdown-document":
        return [
          "# Authentication Architecture v2",
          "",
          "## Summary",
          "",
          "인증 상태를 서버 세션으로 일원화한다.",
          "",
          "## Problem",
          "",
          "각 클라이언트가 토큰 갱신 로직을 별도로 관리한다.",
          "",
          "## Proposal",
          "",
          "서버 세션으로 전환한다.",
        ].join("\n");
      case "json-suggestions":
        return JSON.stringify([
          {
            ruleId: "C001",
            severity: "warn",
            section: "Summary",
            message: "핵심 결론이 너무 늦게 등장한다.",
          },
        ]);
      case "json-merge":
        return JSON.stringify({
          markdown: "# Auth\n\n## Summary\n\n서버 세션과 refresh token을 함께 사용한다.",
          summary: "merged both refresh strategies",
          unresolved: [{ section: "Decision", reason: "만료 정책이 서로 모순됨" }],
        });
    }
  });
}

function makeEngine() {
  return new WritingEngine({
    adapter: makeAdapter(),
    registry,
    rules: COGNITIVE_RULES,
  });
}

describe("WritingEngine", () => {
  it("extracts document intent before composing (§7)", async () => {
    const intent = await makeEngine().extractIntent("메모...");
    expect(intent.intent).toBe("DESIGN_PROPOSAL");
    expect(intent.motivation).toHaveLength(2);
    expect(intent.unknowns).toEqual(["migration strategy"]);
  });

  it("composes a structured document as a Proposal, never a commit (§10, §11)", async () => {
    const result = await makeEngine().compose({
      notes: "토큰을 서버 세션으로 바꾸자...",
      typeId: "design",
      init: { id: "auth-v2", authors: ["minmin"] },
    });
    expect(result.document.title).toBe("Authentication Architecture v2");
    expect(result.document.sections.map((s) => s.role)).toEqual(["summary", "problem", "proposal"]);
    expect(result.proposal.before).toBe("");
    expect(result.proposal.after).toContain("id: auth-v2");
    expect(result.proposal.after).toContain("type: design");
    expect(result.proposal.diff).toContain("+# Authentication Architecture v2");
    // the proposal's markdown must itself be parseable — open format guarantee
    const reparsed = parseDocument(result.proposal.after, registry);
    expect(reparsed.metadata.authors).toEqual(["minmin"]);
  });

  it("rewrites while keeping metadata intact", async () => {
    const { document } = await makeEngine().compose({
      notes: "n",
      typeId: "design",
      init: { id: "auth-v2" },
    });
    const proposal = await makeEngine().rewrite({ document, goal: "더 간결하게" });
    expect(proposal.task).toBe("rewrite");
    expect(proposal.after).toContain("id: auth-v2");
    expect(proposal.summary).toBe("더 간결하게");
  });

  it("critiques without modifying the document (§10)", async () => {
    const { document } = await makeEngine().compose({
      notes: "n",
      typeId: "design",
      init: { id: "auth-v2" },
    });
    const suggestions = await makeEngine().critique({ document });
    expect(suggestions).toHaveLength(1);
    expect(suggestions[0].ruleId).toBe("C001");
    expect(suggestions[0].severity).toBe("warn");
  });

  it("surfaces unresolved contradictions instead of deciding them (§16)", async () => {
    const proposal = await makeEngine().merge({
      path: "projects/app/documents/auth.md",
      base: "---\nid: auth\ntype: design\n---\n\n# Auth\n\n## Summary\n\n기존.",
      current: "---\nid: auth\ntype: design\n---\n\n# Auth\n\n## Summary\n\n서버 세션.",
      incoming: "---\nid: auth\ntype: design\n---\n\n# Auth\n\n## Summary\n\nrefresh token.",
    });
    expect(proposal.unresolved).toEqual([{ section: "Decision", reason: "만료 정책이 서로 모순됨" }]);
    // metadata comes from CURRENT, not from the AI
    expect(proposal.after).toContain("id: auth");
    expect(proposal.after).toContain("서버 세션과 refresh token");
  });
});

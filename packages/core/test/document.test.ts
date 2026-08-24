import { describe, expect, it } from "vitest";
import {
  DocumentTypeRegistry,
  DESIGN_TYPE,
  createSkeleton,
  parseDocument,
  serializeDocument,
} from "@adoc/core";

const SAMPLE = `---
id: auth-v2
type: design
status: accepted
authors:
  - minmin
relations:
  - type: implements
    target: github:load28/app#124
  - type: supersedes
    target: document:auth-v1
---

# Authentication Architecture v2

## Summary

인증 상태를 서버 세션으로 일원화한다.

## Problem

현재 각 클라이언트가 토큰 갱신 로직을 별도로 관리한다.

## Proposal

서버 세션 기반으로 전환한다.

\`\`\`ts
## this heading-looking line lives inside a fence
const x = 1;
\`\`\`

## 회고

자유 형식 섹션도 유지되어야 한다.
`;

describe("document parse/serialize", () => {
  const registry = new DocumentTypeRegistry();

  it("parses frontmatter metadata and semantic sections", () => {
    const doc = parseDocument(SAMPLE, registry);
    expect(doc.id).toBe("auth-v2");
    expect(doc.type).toBe("design");
    expect(doc.title).toBe("Authentication Architecture v2");
    expect(doc.metadata.status).toBe("accepted");
    expect(doc.metadata.authors).toEqual(["minmin"]);
    expect(doc.metadata.relations).toEqual([
      { type: "implements", target: "github:load28/app#124" },
      { type: "supersedes", target: "document:auth-v1" },
    ]);

    const roles = doc.sections.map((s) => s.role);
    expect(roles).toEqual(["summary", "problem", "proposal", "custom"]);
    // heading inside code fence must not split sections
    expect(doc.sections[2].content).toContain("const x = 1;");
    expect(doc.sections[3].heading).toBe("회고");
  });

  it("round-trips through serialize → parse", () => {
    const doc = parseDocument(SAMPLE, registry);
    const serialized = serializeDocument(doc);
    const reparsed = parseDocument(serialized, registry);
    expect(reparsed.title).toBe(doc.title);
    expect(reparsed.metadata).toEqual(doc.metadata);
    expect(reparsed.sections).toEqual(doc.sections);
  });

  it("creates a skeleton matching the type structure", () => {
    const doc = createSkeleton(DESIGN_TYPE, { id: "x", title: "X Design" });
    expect(doc.sections.map((s) => s.role)).toEqual(DESIGN_TYPE.sections.map((s) => s.role));
    const reparsed = parseDocument(serializeDocument(doc), new DocumentTypeRegistry());
    expect(reparsed.sections.map((s) => s.role)).toEqual(DESIGN_TYPE.sections.map((s) => s.role));
  });
});

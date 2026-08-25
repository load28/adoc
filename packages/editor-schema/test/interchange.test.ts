import { describe, expect, test } from "bun:test";

import { exportDocumentText, importDocumentText } from "../src";

const ids = Array.from(
  { length: 20 },
  (_, index) => `00000000-0000-4000-8000-${String(index + 1).padStart(12, "0")}`,
);

describe("document interchange", () => {
  test("imports supported Markdown structures and exports them deterministically", () => {
    let index = 0;
    const content = importDocumentText(
      "# 제목\n\n- 항목\n- [x] 완료\n\n```rust\nfn main() {}\n```\n\n<script>alert(1)</script>",
      "markdown",
      () => requiredId(ids[index++]),
    );
    expect(content.root.children.map((block) => block.type)).toEqual([
      "heading",
      "bulletList",
      "taskList",
      "codeBlock",
      "paragraph",
    ]);
    expect(exportDocumentText(content, "markdown")).toContain("<script>alert(1)</script>");
  });

  test("round-trips plain text without executing or interpreting markup", () => {
    let index = 0;
    const source = "첫 문단\n줄바꿈\n\n<script>문자열</script>";
    const content = importDocumentText(source, "plain", () => requiredId(ids[index++]));
    expect(exportDocumentText(content, "plain")).toBe(source);
  });
});

function requiredId(value: string | undefined): string {
  if (!value) throw new Error("test id fixture exhausted");
  return value;
}

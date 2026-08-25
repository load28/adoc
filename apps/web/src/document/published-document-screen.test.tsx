import { describe, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";

import { ProductAppProvider } from "../shell/product-app-provider";
import { UnpublishedDocumentState } from "./published-document-screen";

describe("published document states", () => {
  test("renders an unpublished document without workspace or placeholder copy", () => {
    const html = renderToStaticMarkup(
      <ProductAppProvider locale="ko" theme="LIGHT">
        <UnpublishedDocumentState title="첫 문서" />
      </ProductAppProvider>,
    );

    expect(html).toContain("첫 문서");
    expect(html).toContain("아직 게시된 버전이 없습니다.");
    expect(html).not.toContain("참여 중인 워크스페이스가 없습니다.");
    expect(html).not.toContain("이 화면의 기능을 준비하고 있습니다.");
  });
});

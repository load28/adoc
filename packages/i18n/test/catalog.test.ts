import { describe, expect, test } from "bun:test";

import { formatInstant, parseLocale, supportedLocales, translate } from "../src";

describe("i18n catalog", () => {
  test("keeps a closed locale registry", () => {
    expect(supportedLocales).toEqual(["ko", "en"]);
    expect(parseLocale("en")).toBe("en");
    expect(parseLocale("ja")).toBe("ko");
  });

  test("requires every key in both catalogs", () => {
    expect(translate("ko", "navigation.home")).toBe("홈");
    expect(translate("en", "navigation.home")).toBe("Home");
  });

  test("formats the same instant in the user timezone", () => {
    expect(formatInstant("ko", "Asia/Seoul", "2026-08-25T00:00:00Z")).toContain("오전 9:00");
  });
});

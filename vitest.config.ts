import { defineConfig } from "vitest/config";
import * as path from "node:path";

export default defineConfig({
  resolve: {
    // Tests run against TS sources directly — no build step needed.
    alias: {
      "@adoc/core": path.resolve(__dirname, "packages/core/src/index.ts"),
      "@adoc/git": path.resolve(__dirname, "packages/git/src/index.ts"),
      "@adoc/indexer": path.resolve(__dirname, "packages/indexer/src/index.ts"),
      "@adoc/node-ports": path.resolve(__dirname, "packages/node-ports/src/index.ts"),
    },
  },
  test: {
    include: ["packages/*/test/**/*.test.ts", "apps/*/test/**/*.test.ts"],
    testTimeout: 30_000,
  },
});

import { extname, posix } from "node:path";

const CONTENT_TYPES: Readonly<Record<string, string>> = {
  ".css": "text/css; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".svg": "image/svg+xml",
  ".woff2": "font/woff2",
};

export type ClientAsset = Readonly<{
  relativePath: string;
  contentType: string;
}>;

export function resolveClientAsset(pathname: string): ClientAsset | undefined {
  let decoded: string;
  try {
    decoded = decodeURIComponent(pathname);
  } catch {
    return undefined;
  }
  if (!decoded.startsWith("/assets/") || decoded.includes("\0")) return undefined;
  const normalized = posix.normalize(decoded);
  if (normalized !== decoded || normalized.includes("..")) return undefined;
  const extension = extname(normalized).toLowerCase();
  return {
    relativePath: `client${normalized}`,
    contentType: CONTENT_TYPES[extension] ?? "application/octet-stream",
  };
}

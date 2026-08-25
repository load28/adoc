const upstreamNamespaces = ["/api/v1/", "/public/v1/"] as const;

export function isApiUpstreamPath(pathname: string): boolean {
  return upstreamNamespaces.some((prefix) => pathname.startsWith(prefix));
}

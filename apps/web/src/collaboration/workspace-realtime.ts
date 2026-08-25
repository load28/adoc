import {
  invalidationRoots,
  type WorkspaceRealtimeEvent,
  workspaceStreamUrl,
  workspaceRealtimeEvents,
} from "@adoc/ui-domain";
import { useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";

export function useWorkspaceRealtime(workspaceId: string) {
  const queryClient = useQueryClient();
  useEffect(() => {
    const storageKey = `adoc.stream.${workspaceId}`;
    const cursor = sessionStorage.getItem(storageKey);
    const query = new URL(workspaceStreamUrl(workspaceId), window.location.origin);
    if (cursor) query.searchParams.set("cursor", cursor);
    const source = new EventSource(`${query.pathname}${query.search}`);
    const listeners = workspaceRealtimeEvents.map((eventName) => {
      const listener = (event: MessageEvent) => {
        if (event.lastEventId) sessionStorage.setItem(storageKey, event.lastEventId);
        for (const root of invalidationRoots(eventName as WorkspaceRealtimeEvent)) {
          void queryClient.invalidateQueries({ queryKey: [root, workspaceId] });
        }
      };
      source.addEventListener(eventName, listener);
      return [eventName, listener] as const;
    });
    return () => {
      for (const [eventName, listener] of listeners)
        source.removeEventListener(eventName, listener);
      source.close();
    };
  }, [queryClient, workspaceId]);
}

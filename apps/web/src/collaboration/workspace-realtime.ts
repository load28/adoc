import {
  invalidationRoots,
  type WorkspaceRealtimeEvent,
  workspaceRealtimeEvents,
} from "@adoc/ui-domain";
import { useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";

export function useWorkspaceRealtime(workspaceId: string) {
  const queryClient = useQueryClient();
  useEffect(() => {
    const storageKey = `adoc.stream.${workspaceId}`;
    const cursor = sessionStorage.getItem(storageKey);
    const query = new URLSearchParams({ workspaceId });
    if (cursor) query.set("cursor", cursor);
    const source = new EventSource(`/api/v1/stream?${query}`);
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

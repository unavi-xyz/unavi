import { ClientSchedules, useClientStore } from "@unavi/react-client";
import { useCallback, useState } from "react";
import { toast } from "react-hot-toast";

import { updateWorld } from "@/app/api/worlds/[id]/helper";
import { usePlayStore } from "@/app/play/store";

export function useSave() {
  const [saving, setSaving] = useState(false);

  const save = useCallback(async () => {
    if (saving) return;

    const { worldId, metadata } = usePlayStore.getState();
    if (worldId.type !== "id") return;

    const { engine } = useClientStore.getState();
    if (!engine) return;

    setSaving(true);

    const toastId = "world-save";
    toast.loading("Saving...", { id: toastId, position: "top-right" });

    try {
      // Save metadata
      await updateWorld(worldId.value, {
        description: metadata.info?.description,
        title: metadata.info?.title,
      });

      // TODO: Export model
      engine.queueSchedule(ClientSchedules.Export);

      toast.success("Saved!", { id: toastId });
    } catch {
      toast.error("Failed to save world", { id: toastId });
    } finally {
      setSaving(false);
    }
  }, [saving]);

  return { save, saving };
}

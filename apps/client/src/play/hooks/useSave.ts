import { ClientSchedules, useClientStore } from "@unavi/react-client";
import { useCallback, useState } from "react";
import { toast } from "react-hot-toast";

import { updateWorld } from "@/app/api/worlds/[id]/helper";
import { getWorldModelFileUpload } from "@/app/api/worlds/[id]/model/files/[file]/helper";
import { createWorldModel } from "@/app/api/worlds/[id]/model/helper";
import { usePlayStore } from "@/app/play/store";

const toastId = "world-save";

export function useSave() {
  const [saving, setSaving] = useState(false);

  const save = useCallback(async () => {
    if (saving) return;

    const { worldId, metadata } = usePlayStore.getState();
    if (worldId.type !== "id") return;

    const { engine } = useClientStore.getState();
    if (!engine) return;

    setSaving(true);

    toast.loading("Saving...", { id: toastId, position: "top-right" });

    engine.queueSchedule(ClientSchedules.Export);

    try {
      // Save metadata
      await updateWorld(worldId.value, {
        description: metadata.info?.description,
        title: metadata.info?.title,
      });

      const image = metadata.info?.image;
      const imageBlob = image
        ? await fetch(image).then((res) => res.blob())
        : null;

      // Create new world model
      await createWorldModel(worldId.value);

      // Get upload URLs
      const [imageUploadURL, modelUploadURL] = await Promise.all([
        getWorldModelFileUpload(worldId.value, "image"),
        getWorldModelFileUpload(worldId.value, "model"),
      ]);

      // Upload image
      if (imageBlob) {
        await fetch(imageUploadURL, {
          body: imageBlob,
          headers: {
            "Content-Type": "image/jpeg",
            "x-amz-acl": "public-read",
          },
          method: "PUT",
        });
      }

      // Wait for export
      await new Promise<void>((resolve) => {
        const interval = setInterval(() => {
          const model = useClientStore.getState().exportedModel;
          if (model) {
            clearInterval(interval);
            resolve();
          }
        }, 200);
      });

      // Upload model
      const modelBlob = useClientStore.getState().exportedModel;
      if (!modelBlob) throw new Error("No model to save");

      await fetch(modelUploadURL, {
        body: modelBlob,
        headers: {
          "Content-Type": "application/gltf+binary",
          "x-amz-acl": "public-read",
        },
        method: "PUT",
      });

      toast.success("Saved world", { id: toastId });
    } catch {
      toast.error("Failed to save world", { id: toastId });
    } finally {
      setSaving(false);
    }
  }, [saving]);

  return { save, saving };
}

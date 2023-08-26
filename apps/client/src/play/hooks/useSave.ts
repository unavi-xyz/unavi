import { EngineSchedules, exportedModelAtom } from "@unavi/engine";
import { getDefaultStore } from "jotai";
import { useCallback, useState } from "react";
import { toast } from "react-hot-toast";

import { updateWorld } from "@/app/api/worlds/[id]/helper";
import { getWorldModelFileUpload } from "@/app/api/worlds/[id]/model/files/[file]/helper";
import { createWorldModel } from "@/app/api/worlds/[id]/model/helper";
import { engineAtom } from "@/app/play/Client";
import { usePlayStore } from "@/app/play/playStore";
import { cdnURL, S3Path } from "@/src/utils/s3Paths";

const toastId = "world-save";

export function useSave() {
  const [saving, setSaving] = useState(false);

  const save = useCallback(async () => {
    if (saving) return;

    const { worldId, metadata } = usePlayStore.getState();
    if (worldId.type !== "id") return;

    const engine = getDefaultStore().get(engineAtom);
    if (!engine) return;

    setSaving(true);

    toast.loading("Saving...", { id: toastId, position: "top-right" });

    engine.queueSchedule(EngineSchedules.Export);

    try {
      // Save metadata
      await updateWorld(worldId.value, {
        description: metadata?.description,
        title: metadata?.title,
      });

      const image = metadata?.image;
      const imageBlob = image
        ? await fetch(image).then((res) => res.blob())
        : null;

      // Create new world model
      const { modelId } = await createWorldModel(worldId.value);

      // Get upload URLs
      const [imageUploadURL, modelUploadURL] = await Promise.all([
        getWorldModelFileUpload(worldId.value, "image"),
        getWorldModelFileUpload(worldId.value, "model"),
      ]);

      // Upload image
      if (imageBlob) {
        const res = await fetch(imageUploadURL, {
          body: imageBlob,
          headers: {
            "Content-Type": "image/jpeg",
            "x-amz-acl": "public-read",
          },
          method: "PUT",
        });

        if (res.ok) {
          setTimeout(() => {
            usePlayStore.setState({
              metadata: {
                ...metadata,
                image: cdnURL(S3Path.worldModel(modelId).image),
              },
            });
          }, 1000);
        }
      }

      // Wait for export
      await new Promise<void>((resolve) => {
        const interval = setInterval(() => {
          const model = getDefaultStore().get(exportedModelAtom);

          if (model) {
            clearInterval(interval);
            resolve();
          }
        }, 200);
      });

      // Upload model
      const modelBlob = getDefaultStore().get(exportedModelAtom);
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

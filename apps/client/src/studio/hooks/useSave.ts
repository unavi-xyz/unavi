import { Packet } from "@gltf-transform/extensions";
import { useCallback, useState } from "react";

import { getProjectFileUpload } from "@/app/api/projects/[id]/files/[file]/helper";
import { updateProject } from "@/app/api/projects/[id]/helper";
import { useAuth } from "@/src/client/AuthProvider";
import { env } from "@/src/env.mjs";

import { useStudio } from "../components/Studio";

export function useSave(projectId: string) {
  const { engine, canvasRef, title, description, setImage, changeMode } = useStudio();
  const { user } = useAuth();

  const [saving, setSaving] = useState(false);

  const saveImage = useCallback(async () => {
    if (!canvasRef.current) return;

    // Take screenshot of canvas
    const image = canvasRef.current.toDataURL("image/jpeg");

    const response = await fetch(image);
    const blob = await response.blob();

    setImage(URL.createObjectURL(blob));

    // Upload to S3
    const url = await getProjectFileUpload(projectId, "image");

    const res = await fetch(url, {
      body: blob,
      headers: { "Content-Type": "image/jpeg" },
      method: "PUT",
    });

    if (!res.ok) throw new Error("Failed to upload image");
  }, [projectId, canvasRef, setImage]);

  const saveModel = useCallback(async () => {
    if (!engine) return;

    // Save XMP metadata
    let xmpPacket = engine.scene.doc.getRoot().getExtension<Packet>(Packet.EXTENSION_NAME);

    if (!xmpPacket) {
      xmpPacket = engine.scene.extensions.xmp
        .createPacket()
        .setContext({ dc: "http://purl.org/dc/elements/1.1/" });

      engine.scene.doc.getRoot().setExtension(xmpPacket.extensionName, xmpPacket);
    }

    const creator = user?.username ? `${user.username}@${env.NEXT_PUBLIC_DEPLOYED_URL}` : undefined;
    const date = new Date().toISOString();

    xmpPacket.setProperty("dc:title", title.trimEnd());
    xmpPacket.setProperty("dc:date", date);
    xmpPacket.setProperty("dc:description", description.trimEnd());
    if (creator) xmpPacket.setProperty("dc:creator", creator);

    // Export to GLB
    const glb = await engine.scene.export({ log: process.env.NODE_ENV === "development" });

    // Upload to S3
    const url = await getProjectFileUpload(projectId, "model");

    const res = await fetch(url, {
      body: glb,
      headers: { "Content-Type": "model/gltf-binary" },
      method: "PUT",
    });

    if (!res.ok) throw new Error("Failed to upload model");
  }, [projectId, description, engine, user, title]);

  const saveMetadata = useCallback(async () => {
    await updateProject(projectId, { description, title });
  }, [projectId, title, description]);

  const save = useCallback(async () => {
    setSaving(true);

    await changeMode("edit");

    try {
      await Promise.all([saveImage(), saveMetadata(), saveModel()]);
    } catch (err) {
      console.error(err);
    }

    setSaving(false);
  }, [saveImage, saveMetadata, saveModel, changeMode]);

  return { save, saveImage, saveMetadata, saveModel, saving };
}

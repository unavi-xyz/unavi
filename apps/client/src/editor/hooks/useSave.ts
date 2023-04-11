import { Packet } from "@gltf-transform/extensions";
import { Space } from "@wired-labs/gltf-extensions";
import { useCallback, useState } from "react";

import { getProjectFileUpload } from "@/app/api/projects/[id]/files/[file]/helper";
import { updateProject } from "@/app/api/projects/[id]/helper";
import { useSession } from "@/src/client/auth/useSession";
import { env } from "@/src/env.mjs";
import { useProfileByAddress } from "@/src/play/hooks/useProfileByAddress";
import { Project } from "@/src/server/helpers/fetchProject";
import { toHex } from "@/src/utils/toHex";

import { useEditor } from "../components/Editor";

export function useSave(project: Project) {
  const { engine, canvasRef, title, setImage, changeMode } = useEditor();
  const { data: session } = useSession();
  const { profile } = useProfileByAddress(session?.address);

  const [saving, setSaving] = useState(false);

  const saveImage = useCallback(async () => {
    if (!canvasRef.current) return;

    // Take screenshot of canvas
    const image = canvasRef.current.toDataURL("image/jpeg");

    const response = await fetch(image);
    const blob = await response.blob();

    setImage(URL.createObjectURL(blob));

    // Upload to S3
    const url = await getProjectFileUpload(project.id, "image");

    const res = await fetch(url, {
      method: "PUT",
      body: blob,
      headers: { "Content-Type": "image/jpeg" },
    });

    if (!res.ok) throw new Error("Failed to upload image");
  }, [project, canvasRef, setImage]);

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

    const creator = profile
      ? `${env.NEXT_PUBLIC_DEPLOYED_URL}/user/${toHex(profile.id)}`
      : session?.address
      ? `${env.NEXT_PUBLIC_DEPLOYED_URL}/user/${session.address}`
      : "";

    const date = new Date().toISOString();

    xmpPacket.setProperty("dc:title", title.trimEnd());
    xmpPacket.setProperty("dc:creator", creator);
    xmpPacket.setProperty("dc:date", date);
    xmpPacket.setProperty("dc:description", project.description.trimEnd());

    // Save space metadata
    let space = engine.scene.doc.getRoot().getExtension<Space>(Space.EXTENSION_NAME);

    if (!space) {
      space = engine.scene.extensions.space.createSpace();
      engine.scene.doc.getRoot().setExtension(space.extensionName, space);
    }

    space.setHost(env.NEXT_PUBLIC_DEFAULT_HOST);

    // Export to GLB
    const glb = await engine.scene.export({ log: process.env.NODE_ENV === "development" });

    // Upload to S3
    const url = await getProjectFileUpload(project.id, "model");

    const res = await fetch(url, {
      method: "PUT",
      body: glb,
      headers: { "Content-Type": "model/gltf-binary" },
    });

    if (!res.ok) throw new Error("Failed to upload model");
  }, [project, engine, profile, session, title]);

  const saveMetadata = useCallback(async () => {
    await updateProject(project.id, { title, description: project.description });
  }, [project, title]);

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

  return { saving, save, saveImage, saveMetadata, saveModel };
}

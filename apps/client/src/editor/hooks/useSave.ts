import { Packet } from "@gltf-transform/extensions";
import { Space } from "@wired-labs/gltf-extensions";

import { getProjectFileUpload } from "@/app/api/projects/[id]/files/[file]/helper";
import { updateProject } from "@/app/api/projects/[id]/helper";
import { useEditorStore } from "@/app/editor/[id]/store";
import { useSession } from "@/src/client/auth/useSession";
import { env } from "@/src/env.mjs";
import { useProfileByAddress } from "@/src/play/hooks/useProfileByAddress";
import { toHex } from "@/src/utils/toHex";

export function useSave(projectId: string) {
  const { data: session } = useSession();
  const { profile } = useProfileByAddress(session?.address);

  async function saveImage() {
    const { engine, canvas } = useEditorStore.getState();
    if (!engine || !canvas) throw new Error("No engine");

    // Take screenshot of canvas
    const image = canvas.toDataURL("image/jpeg");
    useEditorStore.setState({ image });

    const response = await fetch(image);
    const body = await response.blob();

    // Upload to S3
    const url = await getProjectFileUpload(projectId, "image");

    const res = await fetch(url, {
      method: "PUT",
      body,
      headers: { "Content-Type": "image/jpeg" },
    });

    if (!res.ok) throw new Error("Failed to upload image");
  }

  async function saveModel() {
    const { engine, title, description } = useEditorStore.getState();
    if (!engine) throw new Error("No engine");

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

    xmpPacket.setProperty("dc:title", title);
    xmpPacket.setProperty("dc:creator", creator);
    xmpPacket.setProperty("dc:date", date);
    xmpPacket.setProperty("dc:description", description);

    // Save space metadata
    let space = engine.scene.doc.getRoot().getExtension<Space>(Space.EXTENSION_NAME);

    if (!space) {
      space = engine.scene.extensions.space.createSpace();
      engine.scene.doc.getRoot().setExtension(space.extensionName, space);
    }

    space.setHost(env.NEXT_PUBLIC_DEFAULT_HOST);

    // Export to GLB
    const glb = await engine.scene.export();

    // Upload to S3
    const url = await getProjectFileUpload(projectId, "model");

    const res = await fetch(url, {
      method: "PUT",
      body: glb,
      headers: { "Content-Type": "model/gltf-binary" },
    });

    if (!res.ok) throw new Error("Failed to upload model");
  }

  async function saveMetadata() {
    const { title, description } = useEditorStore.getState();
    await updateProject(projectId, { title, description });
  }

  async function save() {
    const { isSaving, sceneLoaded, stopPlaying } = useEditorStore.getState();
    if (isSaving || !sceneLoaded) return;

    useEditorStore.setState({ isSaving: true });

    await stopPlaying();

    try {
      await Promise.all([saveImage(), saveMetadata(), saveModel()]);
    } catch (err) {
      console.error(err);
    }

    useEditorStore.setState({ isSaving: false });
  }

  return { save, saveImage, saveMetadata, saveModel };
}

import { useSearchParams } from "next/navigation";

import { getProjectFileUpload } from "../../../app/api/projects/[id]/[file]/helper";
import { updateProject } from "../../../app/api/projects/[id]/helper";
import { useEditorStore } from "../store";

export function useSave() {
  const params = useSearchParams();
  const id = params?.get("id");

  async function saveImage() {
    if (!id) return;

    const { engine, canvas } = useEditorStore.getState();
    if (!engine || !canvas) throw new Error("No engine");

    // Take screenshot of canvas
    const image = canvas.toDataURL("image/jpeg");
    useEditorStore.setState({ image });

    const response = await fetch(image);
    const body = await response.blob();

    // Upload to S3
    const url = await getProjectFileUpload(id, "image");

    const res = await fetch(url, {
      method: "PUT",
      body,
      headers: { "Content-Type": "image/jpeg" },
    });

    if (!res.ok) throw new Error("Failed to upload image");
  }

  async function saveModel() {
    if (!id) return;

    const { engine } = useEditorStore.getState();
    if (!engine) throw new Error("No engine");

    // Export to GLB
    const glb = await engine.scene.export();

    // Upload to S3
    const url = await getProjectFileUpload(id, "model");

    const res = await fetch(url, {
      method: "PUT",
      body: glb,
      headers: { "Content-Type": "model/gltf-binary" },
    });

    if (!res.ok) throw new Error("Failed to upload model");
  }

  async function saveMetadata() {
    if (!id) return;

    const { name, description } = useEditorStore.getState();

    await updateProject(id, { name, description });
  }

  async function save() {
    const { isSaving, sceneLoaded, stopPlaying } = useEditorStore.getState();
    if (isSaving || !sceneLoaded) return;

    useEditorStore.setState({ isSaving: true });

    await stopPlaying();

    try {
      await Promise.all([saveImage(), saveModel(), saveMetadata()]);
    } catch (err) {
      console.error(err);
    }

    useEditorStore.setState({ isSaving: false });
  }

  return { save, saveImage, saveMetadata, saveModel };
}

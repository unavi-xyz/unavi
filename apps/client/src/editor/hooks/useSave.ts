import { useRouter } from "next/router";

import { trpc } from "../../client/trpc";
import { useEditorStore } from "../store";

export function useSave() {
  const router = useRouter();
  const id = router.query.id as string;

  const { mutateAsync: getImageUpload } = trpc.project.getImageUpload.useMutation();
  const { mutateAsync: getModelUpload } = trpc.project.getModelUpload.useMutation();
  const { mutateAsync: update } = trpc.project.update.useMutation();

  const utils = trpc.useContext();

  async function saveImage() {
    const { engine, canvas } = useEditorStore.getState();
    if (!engine || !canvas) throw new Error("No engine");

    // Take screenshot of canvas
    const image = canvas.toDataURL("image/jpeg");
    const response = await fetch(image);
    const body = await response.blob();

    // Upload to S3
    const url = await getImageUpload({ id });
    const res = await fetch(url, {
      method: "PUT",
      body,
      headers: { "Content-Type": "image/jpeg" },
    });

    if (!res.ok) throw new Error("Failed to upload image");

    utils.project.image.invalidate({ id });
    utils.project.get.invalidate({ id });
    utils.project.getAll.invalidate();
  }

  async function saveModel() {
    const { engine } = useEditorStore.getState();
    if (!engine) throw new Error("No engine");

    // Export to GLB
    const glb = await engine.modules.scene.export();

    // Upload to S3
    const url = await getModelUpload({ id });
    const res = await fetch(url, {
      method: "PUT",
      body: glb,
      headers: { "Content-Type": "model/gltf-binary" },
    });

    if (!res.ok) throw new Error("Failed to upload model");
  }

  async function saveMetadata() {
    const { name, description } = useEditorStore.getState();
    await update({ id, name, description });

    utils.project.get.invalidate({ id });
    utils.project.getAll.invalidate();
  }

  async function save() {
    const { isSaving, sceneLoaded } = useEditorStore.getState();
    if (isSaving || !sceneLoaded) return;

    useEditorStore.setState({ isSaving: true });

    try {
      await Promise.all([saveImage(), saveModel(), saveMetadata()]);
      useEditorStore.setState({ changesToSave: false });
    } catch (err) {
      console.error(err);
    }

    useEditorStore.setState({ isSaving: false });
  }

  return { save, saveImage, saveMetadata, saveModel };
}

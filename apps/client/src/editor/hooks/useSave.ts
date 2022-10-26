import { useRouter } from "next/router";

import { trpc } from "../../client/trpc";
import { useEditorStore } from "../store";
import { SavedSceneJSON } from "../types";
import { imageStorageKey, modelStorageKey } from "../utils/fileStorage";
import { getEditorState } from "../utils/getEditorState";

export function useSave() {
  const router = useRouter();
  const id = router.query.id as string;

  const { mutateAsync: saveProject } = trpc.auth.saveProject.useMutation();
  const { mutateAsync: getFileUpload } =
    trpc.auth.projectFileUploadURL.useMutation();
  const { mutateAsync: getImageUpload } =
    trpc.auth.projectImageUploadURL.useMutation();
  const { mutateAsync: getSceneUpload } =
    trpc.auth.projectSceneUploadURL.useMutation();

  async function saveImage() {
    const { engine, canvas } = useEditorStore.getState();
    if (!engine || !canvas) throw new Error("No engine");

    const image = canvas.toDataURL("image/jpeg");
    const response = await fetch(image);
    const body = await response.blob();

    const url = await getImageUpload({ id });
    const res = await fetch(url, {
      method: "PUT",
      body,
      headers: {
        "Content-Type": "image/jpeg",
      },
    });

    if (!res.ok) throw new Error("Failed to upload image");
  }

  async function uploadFile(
    uri: string,
    fileId: string,
    type: "model" | "image"
  ) {
    const uriResponse = await fetch(uri);
    const buffer = await uriResponse.arrayBuffer();
    const array = new Uint8Array(buffer);

    const storageKey =
      type === "model" ? modelStorageKey(fileId) : imageStorageKey(fileId);

    const url = await getFileUpload({ id, storageKey });

    const response = await fetch(url, {
      method: "PUT",
      body: array,
      headers: {
        "Content-Type": "application/octet-stream",
      },
    });

    if (!response.ok) throw new Error("Failed to upload file");
  }

  async function uploadImageFile(imageId: string) {
    const { engine } = useEditorStore.getState();
    if (!engine) throw new Error("No engine");

    const image = engine.scene.images[imageId];
    if (!image) throw new Error("No image");

    if (!image.isInternal) {
      const url = await getFileUpload({
        id,
        storageKey: imageStorageKey(imageId),
      });

      const response = await fetch(url, {
        method: "PUT",
        body: image.array,
        headers: {
          "Content-Type": image.mimeType,
        },
      });

      if (!response.ok) throw new Error("Failed to upload file");
    }
  }

  async function save() {
    const { name, description, engine } = useEditorStore.getState();
    if (!engine) throw new Error("No engine");

    const promises: Promise<any>[] = [];

    const editorState = getEditorState();
    const scene = engine.scene.toJSON();

    // Save project
    promises.push(
      saveProject({
        id,
        name,
        description,
        editorState,
      })
    );

    // Upload image to S3
    promises.push(saveImage());

    // Upload scene to S3
    promises.push(
      new Promise<void>((resolve, reject) => {
        async function upload() {
          const url = await getSceneUpload({ id });

          const savedScene: SavedSceneJSON = {
            ...scene,
            spawn: scene.spawn ?? [0, 0, 0],
            images: scene.images.map((image) => ({
              id: image.id,
              mimeType: image.mimeType,
            })),
          };

          const body = JSON.stringify(savedScene);

          const res = await fetch(url, {
            method: "PUT",
            body,
            headers: {
              "Content-Type": "application/json",
            },
          });

          if (res.ok) resolve();
          else reject();
        }

        upload();
      })
    );

    // Upload files to S3
    scene.entities.forEach((entity) => {
      // glTF models
      if (entity.mesh?.type === "glTF") {
        const uri = entity.mesh.uri;
        if (uri) promises.push(uploadFile(uri, entity.id, "model"));
      }

      // Images
      if (entity.materialId) {
        const material = engine.scene.materials[entity.materialId];
        if (!material) throw new Error("No material");

        const colorTextureId = material.colorTexture?.imageId;
        if (colorTextureId) promises.push(uploadImageFile(colorTextureId));
      }
    });

    await Promise.all(promises);
  }

  return { save, saveImage };
}

import { useRouter } from "next/router";

import { trpc } from "../../client/trpc";
import { useEditorStore } from "../store";
import { SavedSceneJSON } from "../types";
import {
  binaryStorageKey as binaryStorageKey,
  imageStorageKey,
} from "../utils/fileStorage";
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

  async function save() {
    const fileIds: string[] = [];

    async function uploadBinaryFile(uri: string, fileId: string) {
      const uriResponse = await fetch(uri);
      const buffer = await uriResponse.arrayBuffer();
      const array = new Uint8Array(buffer);

      const storageKey = binaryStorageKey(fileId);
      fileIds.push(storageKey);

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

      const storageKey = imageStorageKey(imageId);
      fileIds.push(storageKey);

      const url = await getFileUpload({ id, storageKey });

      const response = await fetch(url, {
        method: "PUT",
        body: image.array,
        headers: {
          "Content-Type": image.mimeType,
        },
      });

      if (!response.ok) throw new Error("Failed to upload file");
    }

    const { name, description, engine, sceneLoaded } =
      useEditorStore.getState();

    // If scene is not loaded, don't save
    if (!sceneLoaded) return;

    if (!engine) throw new Error("No engine");
    useEditorStore.setState({ changesToSave: false });
    const promises: Promise<any>[] = [];

    const editorState = getEditorState();
    const scene = engine.scene.toJSON();

    // Upload image to S3
    promises.push(saveImage());

    // Upload scene to S3
    promises.push(
      new Promise<void>((resolve, reject) => {
        async function upload() {
          const url = await getSceneUpload({ id });

          const savedScene: SavedSceneJSON = {
            ...scene,
            images: scene.images.map((image) => ({
              id: image.id,
              mimeType: image.mimeType,
            })),
            accessors: scene.accessors.map((accessor) => ({
              ...accessor,
              array: Array.from(accessor.array) as any,
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

    // Upload models to S3
    scene.meshes.forEach((mesh) => {
      if (mesh?.type === "glTF") {
        const uri = mesh.uri;
        if (uri) promises.push(uploadBinaryFile(uri, mesh.id));
      }
    });

    // Upload images to S3
    scene.images.forEach((image) => promises.push(uploadImageFile(image.id)));

    await Promise.all(promises);

    // Save project
    await saveProject({
      id,
      name,
      description,
      editorState,
      fileIds,
    });
  }

  return { save, saveImage };
}

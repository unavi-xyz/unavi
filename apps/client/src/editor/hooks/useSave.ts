import { useRouter } from "next/router";

import { trpc } from "../../client/trpc";
import { useEditorStore } from "../store";
import { getEditorState } from "../utils/getEditorState";

export function useSave() {
  const router = useRouter();

  const { mutateAsync: saveProject } = trpc.useMutation("auth.save-project");
  const { mutateAsync: getFileUpload } = trpc.useMutation(
    "auth.project-file-upload"
  );
  const { mutateAsync: getImageUpload } = trpc.useMutation(
    "auth.project-image-upload"
  );
  const { mutateAsync: getSceneUpload } = trpc.useMutation(
    "auth.project-scene-upload"
  );

  async function save() {
    const { name, description, engine, canvas } = useEditorStore.getState();
    if (!engine) throw new Error("No engine");

    const promises: Promise<any>[] = [];

    const id = router.query.id as string;
    const editorState = JSON.stringify(getEditorState());
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
    promises.push(
      new Promise<void>((resolve, reject) => {
        async function upload() {
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

          if (res.ok) resolve();
          else reject();
        }

        upload();
      })
    );

    // Upload scene to S3
    promises.push(
      new Promise<void>((resolve, reject) => {
        async function upload() {
          const url = await getSceneUpload({ id });
          const res = await fetch(url, {
            method: "PUT",
            body: JSON.stringify(scene),
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

    // Get all files
    const uris: { fileId: string; uri: string }[] = [];
    scene.entities.forEach((entity) => {
      if (entity.mesh?.type === "glTF") {
        const uri = entity.mesh.uri;

        if (uri) {
          uris.push({
            fileId: entity.id,
            uri,
          });
        }
      }
    });

    // Upload files to S3
    const fileUploads = uris.map(async ({ fileId, uri }) => {
      const uriResponse = await fetch(uri);
      const buffer = await uriResponse.arrayBuffer();
      const array = new Uint8Array(buffer);
      const url = await getFileUpload({ id, fileId });
      const response = await fetch(url, {
        method: "PUT",
        body: array,
        headers: {
          "Content-Type": "application/octet-stream",
        },
      });
      if (!response.ok) throw new Error("Failed to upload file");
    });

    promises.push(...fileUploads);

    await Promise.all(promises);
  }

  return { save };
}

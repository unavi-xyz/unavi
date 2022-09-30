import { useRouter } from "next/router";

import { trpc } from "../../auth/trpc";
import { useEditorStore } from "../store";
import { getEditorState } from "../utils/getEditorState";

export function useSave() {
  const router = useRouter();

  const { mutateAsync } = trpc.useMutation("auth.save-project");

  async function save() {
    const { name, description, image, engine } = useEditorStore.getState();
    if (!engine) return;

    const id = router.query.id as string;
    const editorState = JSON.stringify(getEditorState());
    const scene = engine.scene.toJSON();

    // Get all gltf files
    const uris: {
      id: string;
      uri: string;
    }[] = [];

    scene.entities.forEach((entity) => {
      if (entity.mesh?.type === "glTF") {
        const uri = entity.mesh.uri;
        if (uri)
          uris.push({
            id: entity.id,
            uri,
          });
      }
    });

    const fileResponses = await Promise.all(uris.map(({ uri }) => fetch(uri)));
    const fileText = await Promise.all(fileResponses.map((r) => r?.text()));

    const files = fileText.map((text, i) => {
      const uri = uris[i];
      if (!uri) throw new Error("URI not found");

      return { id: uri.id, text };
    });

    await mutateAsync({
      id,
      name,
      description,
      image,
      editorState,
      scene,
      files,
    });
  }

  return { save };
}

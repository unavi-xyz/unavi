import { useEffect, useState } from "react";

import { useEditorStore } from "@/app/editor/[id]/store";

import { Project } from "../../server/helpers/fetchProject";
import { parseError } from "../utils/parseError";

export const ERROR_NOT_SIGNED_IN = "You must be signed in to edit a project.";

export function useLoad(project: Project) {
  const engine = useEditorStore((state) => state.engine);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!engine || !project) return;

    useEditorStore.setState({
      title: project.title,
      description: project.description ?? "",
    });

    return () => {
      useEditorStore.setState({ title: "", description: "" });
    };
  }, [engine, project]);

  useEffect(() => {
    async function load() {
      if (!engine) return;

      const res = await fetch(project.model);
      const buffer = await res.arrayBuffer();
      const array = new Uint8Array(buffer);

      try {
        engine.scene.baseURI = project.model.split("/").slice(0, -1).join("/");
        await engine.scene.addBinary(array);

        // Wait to let scene to load
        await new Promise((resolve) => setTimeout(resolve, 500));
      } catch (err) {
        console.error(err);
        setError(parseError(err, "Failed to load project."));
      }

      useEditorStore.setState({ sceneLoaded: true });
    }

    load();

    return () => {
      engine?.reset();
      useEditorStore.setState({ sceneLoaded: false });
      setError("");
    };
  }, [engine, project]);

  return { error };
}

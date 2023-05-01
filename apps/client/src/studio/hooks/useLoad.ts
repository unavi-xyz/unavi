import { Engine } from "engine";
import { useCallback, useState } from "react";

import { Project } from "../../server/helpers/fetchProject";
import { parseError } from "../utils/parseError";

export const ERROR_NOT_SIGNED_IN = "You must be signed in to edit a project.";

export function useLoad(engine: Engine | null) {
  const [error, setError] = useState<string | null>(null);
  const [loaded, setLoaded] = useState(false);

  const load = useCallback(
    async (project: Project) => {
      setLoaded(false);
      setError(null);

      if (!engine) return;

      try {
        const res = await fetch(project.model);
        const buffer = await res.arrayBuffer();
        const array = new Uint8Array(buffer);

        engine.scene.baseURI = project.model.split("/").slice(0, -1).join("/");
        await engine.scene.addBinary(array);

        // Wait to let scene to load
        await new Promise((resolve) => setTimeout(resolve, 3000));

        setLoaded(true);
      } catch (err) {
        console.error(err);
        setError(parseError(err, "Failed to load project."));
      }
    },
    [engine]
  );

  return { error, loaded, load };
}

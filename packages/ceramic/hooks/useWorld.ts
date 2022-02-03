import { useContext, useEffect, useState } from "react";
import { CeramicContext, Scene } from "..";

export function useWorld(id: string) {
  const { ceramic, authenticated, loader } = useContext(CeramicContext);

  const [world, setWorld] = useState<Scene>();

  useEffect(() => {
    if (!authenticated || !id) return;

    async function get() {
      const doc = await loader.load(id);
      setWorld(doc.content as Scene);
    }

    get();
  }, [authenticated, ceramic, id]);

  return world;
}

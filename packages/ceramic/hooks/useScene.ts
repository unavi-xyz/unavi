import { useContext, useEffect, useState } from "react";
import { CeramicContext, Scene } from "..";

export function useScene(id: string) {
  const { ceramic, authenticated, loader } = useContext(CeramicContext);

  const [scene, setScene] = useState<Scene>();

  useEffect(() => {
    if (!authenticated || !id) return;

    async function get() {
      const doc = await loader.load(id);
      setScene(doc.content as Scene);
    }

    get();
  }, [authenticated, ceramic, id]);

  return scene;
}

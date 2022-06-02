import { useEffect, useState } from "react";

import { useStudioStore } from "../store";

export function useAssetName(assetId: string) {
  const [name, setName] = useState<string>();

  useEffect(() => {
    if (!assetId) {
      setName(undefined);
      return;
    }

    useStudioStore
      .getState()
      .getAssetFileName(assetId)
      .then((res) => {
        if (res) {
          const name = res.split(".").slice(0, -1).join(".");
          setName(name);
        } else {
          setName(undefined);
        }
      })
      .catch((err) => {
        console.error(err);
        setName(undefined);
      });
  }, [assetId]);

  return name;
}

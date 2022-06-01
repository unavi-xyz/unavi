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
        setName(res);
      })
      .catch((err) => {
        console.error(err);
        setName(undefined);
      });
  }, [assetId]);

  return name;
}

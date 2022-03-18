import { useEffect, useState } from "react";
import { getLocalSceneIds } from "./db";

export function useLocalSceneIds() {
  const [ids, setIds] = useState<string[]>([]);

  useEffect(() => {
    getLocalSceneIds().then(setIds);
  }, []);

  return ids;
}

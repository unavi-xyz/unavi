import { useEffect, useState } from "react";

import { getLocalSpace } from "../helpers";
import { LocalSpace } from "../types";

export function useLocalSpace(id: string | undefined) {
  const [localSpace, setLocalSpace] = useState<LocalSpace>();

  useEffect(() => {
    if (!id) {
      setLocalSpace(undefined);
      return;
    }

    getLocalSpace(id)
      .then(setLocalSpace)
      .catch((err) => {
        console.error(err);
        setLocalSpace(undefined);
      });
  }, [id]);

  return localSpace;
}

import { useEffect, useState } from "react";

import { getLocalSpaces } from "../helpers";
import { LocalSpace } from "../types";

export function useLocalSpaces() {
  const [localSpaces, setLocalSpaces] = useState<LocalSpace[]>();

  useEffect(() => {
    getLocalSpaces()
      .then(setLocalSpaces)
      .catch((err) => {
        console.error(err);
        setLocalSpaces([]);
      });
  }, []);

  return localSpaces;
}

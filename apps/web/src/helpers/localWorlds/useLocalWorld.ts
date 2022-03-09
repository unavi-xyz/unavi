import { useEffect, useState } from "react";

import { getLocalWorld } from "./db";
import { LocalWorld } from "./types";

export default function useLocalWorld(id: string) {
  const [world, setWorld] = useState<LocalWorld>();

  useEffect(() => {
    getLocalWorld(id)
      .then((res) => {
        setWorld(res);
      })
      .catch(() => {});
  }, [id]);

  return world;
}

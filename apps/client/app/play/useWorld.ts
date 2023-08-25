import { useEffect, useState } from "react";
import { World } from "thyseus";

export function useWorld() {
  const [world, setWorld] = useState<World | null>(null);

  useEffect(() => {
    import("./world")
      .then(({ resetWorld }) => resetWorld())
      .then((newWorld) => setWorld(newWorld));

    return () => {
      setWorld(null);
    };
  }, []);

  return world;
}

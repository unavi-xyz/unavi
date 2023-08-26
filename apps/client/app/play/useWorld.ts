import { useEffect, useState } from "react";
import { World } from "thyseus";

export function useWorld() {
  const [world, setWorld] = useState<World | null>(null);

  useEffect(() => {
    return () => {
      setWorld(null);
    };
  }, []);

  return world;
}

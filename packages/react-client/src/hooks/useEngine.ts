import { Engine } from "lattice-engine/core";
import { useEffect, useState } from "react";
import { World } from "thyseus";

export function useEngine(world: World | null) {
  const [engine, setEngine] = useState<Engine | null>(null);

  useEffect(() => {
    if (!world) return;

    const newEngine = new Engine(world);
    setEngine(newEngine);

    newEngine.start();

    return () => {
      newEngine.destroy();
      setEngine(null);
    };
  }, [world]);

  return engine;
}

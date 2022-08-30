import { addEntity, removeEntity } from "bitecs";
import { useEffect } from "react";

import { useStudioStore } from "../store";

// There is a weird bug with saving, hard to describe
// this somehow fixes it???
export function useFixSave() {
  const engine = useStudioStore((state) => state.engine);

  useEffect(() => {
    if (!engine) return;
    const eid = addEntity(engine.world);
    removeEntity(engine.world, eid);
  }, [engine]);
}

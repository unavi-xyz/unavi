import { WorldBuilder } from "thyseus";

import { setSkybox } from "./systems/setSkybox";
import { setWorld } from "./systems/setWorld";

export function clientPlugin(builder: WorldBuilder) {
  builder.addSystems(setSkybox, setWorld);
}

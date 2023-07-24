import { WorldBuilder } from "thyseus";

import { clientPlugin } from "./client/plugin";
import { editorPlugin } from "./editor/plugin";

export function enginePlugin(builder: WorldBuilder) {
  builder.addPlugin(clientPlugin).addPlugin(editorPlugin);
}

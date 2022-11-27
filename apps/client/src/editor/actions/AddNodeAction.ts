import { Node } from "engine";

import { useEditorStore } from "../store";

export class AddNodeAction {
  constructor(node: Node) {
    const { engine } = useEditorStore.getState();
    if (!engine) throw new Error("Engine not found");

    // Add node
    engine.scene.addNode(node);

    useEditorStore.setState({ changesToSave: true });
  }
}

export function addNode(node: Node) {
  return new AddNodeAction(node);
}

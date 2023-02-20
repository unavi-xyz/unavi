import { NodeExtras } from "engine";

import { useEditorStore } from "../store";
import { useNodeAttribute } from "./useNodeAttribute";
import { useNodes } from "./useNodes";

export function useScript(id: string) {
  const nodes = useNodes();

  const node = nodes.find((node) => {
    const extras = node.getExtras() as NodeExtras;
    return extras.scripts?.find((script) => script.id === id);
  });

  const engine = useEditorStore((state) => state.engine);

  const nodeId = node ? engine?.scene.node.getId(node) : null;

  const extras = useNodeAttribute(nodeId, "extras");

  const scripts = extras?.scripts || [];

  const script = scripts.find((script) => script.id === id);

  return script;
}

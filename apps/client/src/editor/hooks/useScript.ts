import { NodeExtras } from "engine";

import { useNodeExtras } from "./useNodeExtras";
import { useNodes } from "./useNodes";

export function useScript(id: string) {
  const nodes = useNodes();

  const node =
    nodes.find((node) => {
      const extras = node.getExtras() as NodeExtras;
      return extras.scripts?.find((script) => script.id === id);
    }) ?? null;

  const extras = useNodeExtras(node);

  const scripts = extras?.scripts || [];
  const script = scripts.find((script) => script.id === id);

  return script;
}

import { NodeExtras } from "@unavi/engine";

import { useNodeExtras } from "./useNodeExtras";
import { useNodes } from "./useNodes";

export function useScriptExtras(id: string | null): ScriptExtras | null {
  const nodes = useNodes();

  const node =
    nodes.find((node) => {
      const extras = node.getExtras() as NodeExtras;
      return extras.scripts?.find((script) => script.id === id);
    }) ?? null;

  const extras = useNodeExtras(node);

  const scripts = extras?.scripts || [];
  const script = scripts.find((script) => script.id === id);

  return script ?? null;
}

export type ScriptExtras = {
  id: string;
  name: string;
};

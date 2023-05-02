import { InputSocketSpecJSON, NodeSpecJSON, OutputSocketSpecJSON } from "@unavi/behave-graph-core";

export function getSockets(
  nodes: NodeSpecJSON[],
  nodeType: string | undefined,
  handleType: "source" | "target" | null
): OutputSocketSpecJSON[] | InputSocketSpecJSON[] | undefined {
  const nodeSpec = nodes.find((node) => node.type === nodeType);
  if (nodeSpec === undefined) return;
  return handleType === "source" ? nodeSpec.outputs : nodeSpec.inputs;
}

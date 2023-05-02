import { Edge } from "reactflow";

export const isHandleConnected = (
  edges: Edge[],
  nodeId: string,
  handleId: string,
  type: "source" | "target",
  excludeConnection?: { sourceId: string; sourceHandle: string }
) => {
  return edges.some((edge) => {
    if (
      excludeConnection &&
      edge.source === excludeConnection.sourceId &&
      edge.sourceHandle === excludeConnection.sourceHandle
    ) {
      return false;
    }

    return edge[type] === nodeId && edge[`${type}Handle`] === handleId;
  });
};

import { useCallback } from "react";
import { useReactFlow } from "reactflow";

import { FlowNodeParamter } from "../types";

export const useChangeNodeData = (id: string) => {
  const instance = useReactFlow();

  return useCallback(
    (key: string, value: FlowNodeParamter) => {
      instance.setNodes((nodes) =>
        nodes.map((n) => {
          if (n.id !== id) return n;
          return {
            ...n,
            data: {
              ...n.data,
              [key]: value,
            },
          };
        })
      );
    },
    [instance, id]
  );
};

import { useEffect } from "react";
import { Edge, Node, useReactFlow } from "reactflow";

import { useStudio } from "../Studio";
import { useScript } from "./Script";
import { isValidConnection } from "./utils/isValidConnection";
import { saveFlow } from "./utils/saveFlow";

interface Props {
  scriptId: string;
  loaded: string | undefined;
  setEdges: React.Dispatch<React.SetStateAction<Edge<any>[]>>;
  nodes: Node[];
  edges: Edge[];
}

export default function SaveFlow({ scriptId, loaded, setEdges, nodes, edges }: Props) {
  const { engine } = useStudio();
  const { variables } = useScript();
  const instance = useReactFlow();

  // Save nodes to engine
  useEffect(() => {
    if (!engine || !loaded || !scriptId || loaded !== scriptId) return;

    const save = () => {
      saveFlow(nodes, edges, engine, scriptId);
    };

    save();

    variables.forEach((variable) => variable.addEventListener("change", save));
    return () => {
      variables.forEach((variable) => variable.removeEventListener("change", save));
    };
  }, [edges, engine, loaded, nodes, scriptId, variables]);

  // Disconnect invalid edges
  useEffect(() => {
    if (!engine || !loaded || !scriptId || loaded !== scriptId) return;

    const fixEdges = () => {
      const newEdges = edges.filter((edge) => {
        if (!edge.sourceHandle || !edge.targetHandle) return false;

        const valid = isValidConnection(
          {
            source: edge.source,
            sourceHandle: edge.sourceHandle,
            target: edge.target,
            targetHandle: edge.targetHandle,
          },
          instance,
          variables
        );

        return valid;
      });

      if (newEdges.length !== edges.length) setEdges(newEdges);
    };

    fixEdges();

    variables.forEach((variable) => variable.addEventListener("change", fixEdges));
    return () => {
      variables.forEach((variable) => variable.removeEventListener("change", fixEdges));
    };
  }, [nodes, edges, engine, instance, loaded, scriptId, setEdges, variables]);

  return null;
}

import { BehaviorVariable } from "@unavi/gltf-extensions";
import React, { Dispatch, SetStateAction, useContext, useMemo, useState } from "react";
import { ReactFlowInstance, XYPosition } from "reactflow";

export type AddNode = (type: string, position: XYPosition) => void;

export type ScriptContextType = {
  addNode: AddNode;
  contextMenuNodeId: string | null;
  reactflow: ReactFlowInstance | null;
  setAddNode: Dispatch<SetStateAction<AddNode>>;
  setContextMenuNodeId: Dispatch<SetStateAction<string | null>>;
  setReactflow: Dispatch<SetStateAction<ReactFlowInstance | null>>;
  setVariables: Dispatch<SetStateAction<BehaviorVariable[]>>;
  variables: BehaviorVariable[];
};

const defaultContext: ScriptContextType = {
  addNode: () => {},
  contextMenuNodeId: null,
  reactflow: null,
  setAddNode: () => {},
  setContextMenuNodeId: () => {},
  setReactflow: () => {},
  setVariables: () => {},
  variables: [],
};

const ScriptContext = React.createContext<ScriptContextType>(defaultContext);

export function useScript() {
  return useContext(ScriptContext);
}

interface Props {
  children: React.ReactNode;
}

export default function Script({ children }: Props) {
  const [addNode, setAddNode] = useState<AddNode>(() => () => {});
  const [contextMenuNodeId, setContextMenuNodeId] = useState<string | null>(null);
  const [reactflow, setReactflow] = useState<ReactFlowInstance | null>(null);
  const [variables, setVariables] = useState<BehaviorVariable[]>([]);

  const value = useMemo(() => {
    return {
      addNode,
      contextMenuNodeId,
      reactflow,
      setAddNode,
      setContextMenuNodeId,
      setReactflow,
      setVariables,
      variables,
    };
  }, [addNode, contextMenuNodeId, reactflow, variables]);

  return <ScriptContext.Provider value={value}>{children}</ScriptContext.Provider>;
}

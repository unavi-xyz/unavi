import "reactflow/dist/style.css";

import * as ContextMenu from "@radix-ui/react-context-menu";
import { nanoid } from "nanoid";
import { MouseEvent, useCallback, useEffect, useRef, useState } from "react";
import ReactFlow, {
  addEdge,
  Background,
  Connection,
  Controls,
  Edge,
  updateEdge,
  useEdgesState,
  useNodesState,
  useReactFlow,
  XYPosition,
} from "reactflow";

import { useStudio } from "../Studio";
import NodeContextMenu from "./NodeContextMenu";
import NodePicker from "./NodePicker";
import SaveFlow from "./SaveFlow";
import { useScript } from "./Script";
import { loadFlow } from "./utils/loadFlow";
import { nodeTypes } from "./utils/nodeTypes";

interface Props {
  scriptId: string;
}

export default function NodeGraph({ scriptId }: Props) {
  const edgeUpdateSuccessful = useRef(true);

  const [nodePickerPosition, setNodePickerPosition] = useState<XYPosition>();
  const [contextMenuType, setContextMenuType] = useState<"node" | "pane">("pane");
  const [loaded, setLoaded] = useState<string>();

  const { engine, mode } = useStudio();
  const { contextMenuNodeId, setAddNode, setVariables } = useScript();

  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);

  const onConnect = useCallback(
    (params: Connection | Edge) => setEdges((eds) => addEdge(params, eds)),
    [setEdges]
  );

  const onNodeContextMenu = useCallback(() => {
    setContextMenuType("node");
  }, []);

  const onPaneContextMenu = useCallback((e: MouseEvent) => {
    const relativePosition = {
      x: e.clientX - e.currentTarget.getBoundingClientRect().left,
      y: e.clientY - e.currentTarget.getBoundingClientRect().top,
    };
    setNodePickerPosition(relativePosition);
    setContextMenuType("pane");
  }, []);

  const onEdgeUpdateStart = useCallback(() => {
    edgeUpdateSuccessful.current = false;
  }, []);

  const onEdgeUpdate = useCallback(
    (oldEdge: Edge, newConnection: Connection) => {
      if (mode === "play") return;
      edgeUpdateSuccessful.current = true;
      setEdges((els) => updateEdge(oldEdge, newConnection, els));
    },
    [setEdges, mode]
  );

  const onEdgeUpdateEnd = useCallback(
    (_: any, edge: Edge) => {
      if (mode === "play") return;
      if (!edgeUpdateSuccessful.current) setEdges((eds) => eds.filter((e) => e.id !== edge.id));
      edgeUpdateSuccessful.current = true;
    },
    [setEdges, mode]
  );

  const addNode = useCallback(
    (type: string, position: XYPosition) =>
      onNodesChange([{ type: "add", item: { id: nanoid(), type, position, data: {} } }]),
    [onNodesChange]
  );

  const deleteNode = useCallback(
    (id: string) => {
      // Remove all edges connected to the node
      setEdges((edges) => edges.filter((edge) => edge.source !== id && edge.target !== id));
      // Remove the node
      onNodesChange([{ type: "remove", id }]);
    },
    [onNodesChange, setEdges]
  );

  useEffect(() => {
    if (!engine) return;
    // Load variables
    const variables = engine.scene.extensions.behavior.listVariables();
    setVariables(variables);
  }, [engine, setVariables]);

  useEffect(() => {
    setAddNode(addNode);
  }, [addNode, setAddNode]);

  useEffect(() => {
    if (!engine || !scriptId) return;

    // Load nodes from engine
    const { nodes, edges } = loadFlow(engine, scriptId);

    setNodes(nodes);
    setEdges(edges);
    setLoaded(scriptId);
  }, [engine, scriptId, setNodes, setEdges]);

  return (
    <div className="h-full">
      <div className="h-full w-full pb-20">
        <ContextMenu.Root
          onOpenChange={(open) => {
            if (!open) setNodePickerPosition(undefined);
          }}
        >
          <ContextMenu.Trigger disabled={mode === "play"}>
            <ReactFlow
              nodeTypes={nodeTypes}
              nodes={nodes}
              edges={edges}
              onNodesChange={onNodesChange}
              onEdgesChange={onEdgesChange}
              onEdgeUpdate={onEdgeUpdate}
              onEdgeUpdateStart={onEdgeUpdateStart}
              onEdgeUpdateEnd={onEdgeUpdateEnd}
              onConnect={onConnect}
              onNodeContextMenu={onNodeContextMenu}
              onEdgeContextMenu={onPaneContextMenu}
              onPaneContextMenu={onPaneContextMenu}
              proOptions={{ hideAttribution: true }}
              fitView
              className="bg-neutral-100"
            >
              {contextMenuType === "node" ? (
                <NodeContextMenu
                  onDelete={() => {
                    if (contextMenuNodeId) deleteNode(contextMenuNodeId);
                  }}
                />
              ) : contextMenuType === "pane" ? (
                <NodePicker position={nodePickerPosition} />
              ) : null}

              <Controls />
              <Background />

              <SaveFlow
                scriptId={scriptId}
                loaded={loaded}
                nodes={nodes}
                edges={edges}
                setEdges={setEdges}
              />

              <StoreInstance />
            </ReactFlow>
          </ContextMenu.Trigger>
        </ContextMenu.Root>
      </div>
    </div>
  );
}

function StoreInstance() {
  const { setReactflow } = useScript();
  const reactflow = useReactFlow();

  useEffect(() => {
    setReactflow(reactflow);
  }, [reactflow, setReactflow]);

  return null;
}

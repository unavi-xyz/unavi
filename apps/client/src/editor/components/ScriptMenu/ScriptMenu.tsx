import "reactflow/dist/style.css";

import * as ContextMenu from "@radix-ui/react-context-menu";
import { nanoid } from "nanoid";
import { MouseEvent, useCallback, useEffect, useRef, useState } from "react";
import { MdClose } from "react-icons/md";
import ReactFlow, {
  addEdge,
  Background,
  Connection,
  Controls,
  Edge,
  updateEdge,
  useEdgesState,
  useNodesState,
  XYPosition,
} from "reactflow";

import IconButton from "../../../ui/IconButton";
import { useScript } from "../../hooks/useScript";
import { useEditorStore } from "../../store";
import NodeContextMenu from "./NodeContextMenu";
import NodePicker from "./NodePicker";
import { loadFlow } from "./utils/loadFlow";
import { nodeTypes } from "./utils/nodeTypes";
import { saveFlow } from "./utils/saveFlow";

interface Props {
  scriptId: string;
}

export default function ScriptMenu({ scriptId }: Props) {
  const edgeUpdateSuccessful = useRef(true);

  const [nodePickerPosition, setNodePickerPosition] = useState<XYPosition>();
  const [contextMenuType, setContextMenuType] = useState<"node" | "pane">("pane");
  const [loaded, setLoaded] = useState<string>();
  const engine = useEditorStore((state) => state.engine);

  const script = useScript(scriptId);
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
      edgeUpdateSuccessful.current = true;
      setEdges((els) => updateEdge(oldEdge, newConnection, els));
    },
    [setEdges]
  );

  const onEdgeUpdateEnd = useCallback(
    (_: any, edge: Edge) => {
      if (!edgeUpdateSuccessful.current) setEdges((eds) => eds.filter((e) => e.id !== edge.id));
      edgeUpdateSuccessful.current = true;
    },
    [setEdges]
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
    if (!engine || !scriptId) return;

    // Load from the engine
    const { nodes, edges } = loadFlow(engine, scriptId);

    setNodes(nodes);
    setEdges(edges);
    setLoaded(scriptId);
  }, [engine, scriptId, setNodes, setEdges]);

  useEffect(() => {
    if (!engine || !loaded || !scriptId || loaded !== scriptId) return;

    // Save to the engine
    saveFlow(nodes, edges, engine, scriptId);
  }, [engine, loaded, nodes, edges, scriptId]);

  if (!script) return null;

  return (
    <div className="h-full">
      <div className="flex w-full items-center justify-between px-4 pb-1.5">
        <div className="text-lg">{script.name}</div>

        <div className="h-8">
          <IconButton
            cursor="pointer"
            onClick={() => useEditorStore.setState({ openScriptId: null })}
          >
            <MdClose />
          </IconButton>
        </div>
      </div>

      <div className="h-full w-full pb-20">
        <ContextMenu.Root
          onOpenChange={(open) => {
            if (!open) setNodePickerPosition(undefined);
          }}
        >
          <ContextMenu.Trigger>
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
                    const { contextMenuNodeId } = useEditorStore.getState();
                    if (contextMenuNodeId) deleteNode(contextMenuNodeId);
                  }}
                />
              ) : contextMenuType === "pane" ? (
                <NodePicker position={nodePickerPosition} addNode={addNode} />
              ) : null}

              <Controls />
              <Background />
            </ReactFlow>
          </ContextMenu.Trigger>
        </ContextMenu.Root>
      </div>
    </div>
  );
}

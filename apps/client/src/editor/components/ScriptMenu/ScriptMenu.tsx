import "reactflow/dist/style.css";

import * as ContextMenu from "@radix-ui/react-context-menu";
import { nanoid } from "nanoid";
import { MouseEvent, useCallback, useRef, useState } from "react";
import { MdClose } from "react-icons/md";
import ReactFlow, {
  addEdge,
  Background,
  Connection,
  Controls,
  Edge,
  Node,
  updateEdge,
  useEdgesState,
  useNodesState,
  XYPosition,
} from "reactflow";

import IconButton from "../../../ui/IconButton";
import { useScript } from "../../hooks/useScript";
import { useEditorStore } from "../../store";
import NodePicker from "./NodePicker";
import { nodeTypes } from "./utils/nodeTypes";

const initialNodes: Node[] = [
  { id: "start", type: "lifecycle/onStart", position: { x: 0, y: 0 }, data: {} },
];

interface Props {
  scriptId: string;
}

export default function ScriptMenu({ scriptId }: Props) {
  const edgeUpdateSuccessful = useRef(true);

  const [nodePickerPosition, setNodePickerPosition] = useState<XYPosition>();

  const script = useScript(scriptId);
  const [nodes, , onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);

  const onConnect = useCallback(
    (params: Connection | Edge) => setEdges((eds) => addEdge(params, eds)),
    [setEdges]
  );

  const onPaneContextMenu = useCallback((e: MouseEvent) => {
    const relativePosition = {
      x: e.clientX - e.currentTarget.getBoundingClientRect().left,
      y: e.clientY - e.currentTarget.getBoundingClientRect().top,
    };
    setNodePickerPosition(relativePosition);
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
      if (!edgeUpdateSuccessful.current) {
        setEdges((eds) => eds.filter((e) => e.id !== edge.id));
      }

      edgeUpdateSuccessful.current = true;
    },
    [setEdges]
  );

  const addNode = useCallback(
    (type: string, position: XYPosition) => {
      onNodesChange([
        {
          type: "add",
          item: {
            id: nanoid(),
            type,
            position,
            data: {},
          },
        },
      ]);
    },
    [onNodesChange]
  );

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
              onPaneContextMenu={onPaneContextMenu}
              proOptions={{ hideAttribution: true }}
              fitView
              className="bg-neutral-100"
            >
              <NodePicker position={nodePickerPosition} onPickNode={addNode} />
              <Controls />
              <Background />
            </ReactFlow>
          </ContextMenu.Trigger>
        </ContextMenu.Root>
      </div>
    </div>
  );
}

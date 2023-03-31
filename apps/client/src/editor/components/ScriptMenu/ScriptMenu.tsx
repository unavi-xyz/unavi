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
  useReactFlow,
  XYPosition,
} from "reactflow";

import { useEditorStore } from "../../../../app/editor/[id]/store";
import { DropdownMenu, DropdownTrigger } from "../../../ui/DropdownMenu";
import IconButton from "../../../ui/IconButton";
import { useScript } from "../../hooks/useScript";
import AddNodeMenu from "./AddNodeMenu";
import NodeContextMenu from "./NodeContextMenu";
import NodePicker from "./NodePicker";
import SaveFlow from "./SaveFlow";
import { loadFlow } from "./utils/loadFlow";
import { nodeTypes } from "./utils/nodeTypes";

interface Props {
  scriptId: string;
}

export default function ScriptMenu({ scriptId }: Props) {
  const edgeUpdateSuccessful = useRef(true);

  const [nodePickerPosition, setNodePickerPosition] = useState<XYPosition>();
  const [contextMenuType, setContextMenuType] = useState<"node" | "pane">("pane");
  const [loaded, setLoaded] = useState<string>();
  const engine = useEditorStore((state) => state.engine);
  const isPlaying = useEditorStore((state) => state.isPlaying);

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
      if (isPlaying) return;
      edgeUpdateSuccessful.current = true;
      setEdges((els) => updateEdge(oldEdge, newConnection, els));
    },
    [setEdges, isPlaying]
  );

  const onEdgeUpdateEnd = useCallback(
    (_: any, edge: Edge) => {
      if (isPlaying) return;
      if (!edgeUpdateSuccessful.current) setEdges((eds) => eds.filter((e) => e.id !== edge.id));
      edgeUpdateSuccessful.current = true;
    },
    [setEdges, isPlaying]
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
    useEditorStore.setState({ variables });
  }, [engine]);

  useEffect(() => {
    useEditorStore.setState({ addNode });
  }, [addNode]);

  useEffect(() => {
    if (!engine || !scriptId) return;

    // Load nodes from engine
    const { nodes, edges } = loadFlow(engine, scriptId);

    setNodes(nodes);
    setEdges(edges);
    setLoaded(scriptId);
  }, [engine, scriptId, setNodes, setEdges]);

  if (!script) return null;

  return (
    <div className="h-full">
      <div className="flex w-full items-center justify-between px-4 pb-1.5">
        <div className="flex items-center space-x-8">
          <div className="text-lg">{script.name}</div>

          <DropdownMenu>
            <DropdownTrigger asChild>
              <button className="rounded-md px-2 py-0.5 text-sm text-neutral-500 transition hover:bg-neutral-200 hover:text-black active:bg-neutral-200">
                Add
              </button>
            </DropdownTrigger>

            <AddNodeMenu />
          </DropdownMenu>
        </div>

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
          <ContextMenu.Trigger disabled={isPlaying}>
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
  const reactflow = useReactFlow();

  useEffect(() => {
    useEditorStore.setState({ reactflow });
  }, [reactflow]);

  return null;
}

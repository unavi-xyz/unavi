import { BehaviorVariable } from "@wired-labs/gltf-extensions";
import { Engine } from "engine";
import { ReactFlowInstance, XYPosition } from "reactflow";
import { create } from "zustand";

export type Tool = "translate" | "rotate" | "scale";

export interface IEditorStore {
  engine: Engine | null;

  canvas: HTMLCanvasElement | null;
  sceneLoaded: boolean;
  isSaving: boolean;
  isPlaying: boolean;
  stopPlaying: () => Promise<void>;

  showColliders: boolean;
  tool: Tool;

  title: string;
  description: string;
  image: string | null;

  treeIds: string[];
  openIds: string[];
  draggingId: string | null;
  selectedId: string | null;

  openScriptId: string | null;
  contextMenuNodeId: string | null;
  variables: BehaviorVariable[];
  addNode: (type: string, position: XYPosition) => void;
  reactflow: ReactFlowInstance | null;
}

export const useEditorStore = create<IEditorStore>(() => ({
  engine: null,
  canvas: null,
  sceneLoaded: false,
  isSaving: false,
  isPlaying: false,
  stopPlaying: async () => {},

  showColliders: false,
  tool: "translate",

  title: "",
  description: "",
  image: null,

  treeIds: [],
  openIds: [],
  draggingId: null,
  selectedId: null,

  openScriptId: null,
  contextMenuNodeId: null,
  variables: [],
  addNode: () => {},
  reactflow: null,
}));

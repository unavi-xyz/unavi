import { Properties, SceneStore } from "3d";
import { Euler, Quaternion, Vector3 } from "three";
import { UseBoundStore, StoreApi } from "zustand";

import { sceneManager } from "./store";
import { Selected, Tool } from "./types";

export interface EditorStore extends SceneStore {
  selected: Selected;
  tool: Tool;
  sceneId: string;
  usingGizmo: boolean;
  previewMode: boolean;
  debugMode: boolean;
}

type useStoreType = UseBoundStore<EditorStore, StoreApi<EditorStore>>;

export class EditorManager {
  useStore: useStoreType;

  constructor(useStore: useStoreType) {
    this.useStore = useStore;
  }

  setSelected(selected: Selected) {
    this.useStore.setState({ selected });
  }

  setTool(tool: Tool) {
    this.useStore.setState({ tool });
  }

  setUsingGizmo(usingGizmo: boolean) {
    this.useStore.setState({ usingGizmo });
  }

  setPreviewMode(previewMode: boolean) {
    this.useStore.setState({ previewMode });
  }

  setSceneId(sceneId: string) {
    this.useStore.setState({ sceneId });
  }

  setDebugMode(debugMode: boolean) {
    this.useStore.setState({ debugMode });
  }

  saveSelected() {
    const { id, ref } = this.useStore.getState().selected;
    const instance = this.useStore.getState().scene.instances[id];

    const position = ref.current.getWorldPosition(new Vector3()).toArray();

    const rotationQuat = ref.current.getWorldQuaternion(new Quaternion());
    const rotationEuler = new Euler().setFromQuaternion(rotationQuat);
    const rotation = rotationEuler.toVector3().toArray();

    const changes: Partial<Properties> = { position, rotation };

    if ("scale" in instance.properties) {
      changes.scale = ref.current.getWorldScale(new Vector3()).toArray();
    }

    sceneManager.editInstance(id, changes);
  }
}

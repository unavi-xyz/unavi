import { Heightmap, Properties, SceneStore } from "3d";
import { Euler, Quaternion, Raycaster, Vector3 } from "three";
import { UseBoundStore, StoreApi } from "zustand";

import { sceneManager } from "../store";
import { Selected, Tool } from "../types";

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

  generateSelectedHeightmap(width: number, numRows: number) {
    const selected = this.useStore.getState().selected;
    const instance = this.useStore.getState().scene.instances[selected.id];
    const originCopy = new Vector3();
    const down = new Vector3(0, -1, 0);
    const data: number[][] = [];

    //shoot a downwards raycast along a grid over top of the instance
    const maxHeight = 10000;

    const origin = new Vector3(...instance.properties.position);
    origin.y += maxHeight;

    const raycaster = new Raycaster(origin.clone(), down, 1, maxHeight * 2);

    for (let x = 0; x < numRows; x++) {
      data[x] = [];

      for (let z = 0; z < numRows; z++) {
        originCopy.copy(origin);

        originCopy.x += (x / numRows) * width - width / 2;
        originCopy.z += (z / numRows) * width - width / 2;

        raycaster.set(originCopy, down);

        const intersections = raycaster.intersectObject(selected.ref.current);
        if (intersections.length > 0) {
          const distance = maxHeight - intersections[0].distance;
          data[x][z] = distance;
        } else {
          data[x][z] = 0;
        }
      }

      data[x].reverse();
    }

    const heightmap: Heightmap = { data, width };
    const id = sceneManager.newInstance("Heightmap");
    sceneManager.editInstance(id, { heightmap });

    return id;
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

import { Engine } from "../Engine";
import { NodeJSON } from "../scene/json";
import { Scene } from "../scene/Scene";

export interface SceneOptions {
  engine: Engine;
}

export class MainScene extends Scene {
  #engine: Engine;

  constructor({ engine }: SceneOptions) {
    super();
    this.#engine = engine;
  }

  override createNode(json: Partial<NodeJSON> = {}) {
    const { id, node } = super.createNode(json);

    this.#engine.toRenderThread({ subject: "create_node", data: { id, json } });

    // Subscribe to node events
    node.addEventListener("change", (e) => {
      const attribute = e.attribute as keyof NodeJSON;
      const json = this.toNodeJSON(node);
      const value = json[attribute];

      this.#engine.toRenderThread({
        subject: "change_node",
        data: { id, attribute, value },
      });
    });

    node.addEventListener("dispose", () => {
      this.#engine.toRenderThread({ subject: "dispose_node", data: { id } });
    });

    return { id, node };
  }
}

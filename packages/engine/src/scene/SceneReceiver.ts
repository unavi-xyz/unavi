import { SceneMessage } from "./messages";
import { Scene } from "./Scene";

export class SceneReceiver extends Scene {
  onmessage({ subject, data }: SceneMessage) {
    switch (subject) {
      case "create_node": {
        const { id, object } = this.node.create(data.json, data.id);
      }
    }
  }
}

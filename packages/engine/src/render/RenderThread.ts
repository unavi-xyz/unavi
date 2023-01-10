import { isSceneMessage } from "../scene/messages";
import { SceneReceiver } from "../scene/SceneReceiver";
import { PostMessage } from "../types";
import { FromRenderMessage, ToRenderMessage } from "./messages";

export class RenderThread {
  #canvas: HTMLCanvasElement | OffscreenCanvas | null = null;

  readonly scene = new SceneReceiver();

  postMessage: PostMessage<FromRenderMessage>;

  constructor(postMessage: PostMessage<FromRenderMessage>) {
    this.postMessage = postMessage;
  }

  onmessage = (event: MessageEvent<ToRenderMessage>) => {
    if (isSceneMessage(event.data)) {
      this.scene.onmessage(event.data);
    }

    const { subject, data } = event.data;

    switch (subject) {
      case "set_canvas": {
        this.#canvas = data;
      }
    }
  };
}

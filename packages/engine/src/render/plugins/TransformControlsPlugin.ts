import { PerspectiveCamera, Scene } from "three";

import { PostMessage } from "../../types";
import { PluginState } from "../RenderWorker";
import { FakePointerEvent } from "../classes/OrbitControls";
import { SceneMapper } from "../classes/SceneMapper";
import { TransformControls } from "../classes/TransformControls";
import { FromRenderMessage, ToRenderMessage } from "../types";
import { Plugin } from "./Plugin";

export class TransformControlsPlugin extends Plugin {
  #target = new EventTarget();
  #mapper: SceneMapper;
  #transformControls: TransformControls;
  #postMessage: PostMessage<FromRenderMessage>;

  constructor(
    camera: PerspectiveCamera,
    mapper: SceneMapper,
    scene: Scene,
    postMessage: PostMessage<FromRenderMessage>,
    state: PluginState
  ) {
    super();

    this.#mapper = mapper;
    this.#postMessage = postMessage;
    this.#transformControls = new TransformControls(camera, this.#target);
    scene.add(this.#transformControls);

    this.#transformControls.addEventListener("mouseDown", () => {
      state.usingTransformControls = true;
    });

    this.#transformControls.addEventListener("mouseUp", () => {
      state.usingTransformControls = false;

      // Send new transform to main thread
      const object = this.#transformControls.object;
      if (!object) throw new Error("No object found");

      const position = object.position;
      const rotation = object.quaternion;
      const scale = object.scale;

      this.#postMessage({
        subject: "set_transform",
        data: {
          position: [position.x, position.y, position.z],
          rotation: [rotation.x, rotation.y, rotation.z, rotation.w],
          scale: [scale.x, scale.y, scale.z],
        },
      });
    });
  }

  onmessage(event: MessageEvent<ToRenderMessage>) {
    const { subject, data } = event.data;

    switch (subject) {
      case "set_transform_target":
        if (data === null) this.#transformControls.detach();
        else {
          const object = this.#mapper.findObject(data);
          if (object) this.#transformControls.attach(object);
          else throw new Error(`Object not found: ${data}`);
        }
        break;
      case "set_transform_mode":
        this.#transformControls.mode = data;
        break;
      case "pointermove":
        const pointerMoveEvent: FakePointerEvent = new CustomEvent("pointermove", { detail: data });
        this.#target.dispatchEvent(pointerMoveEvent);
        break;
      case "pointerdown":
        const pointerDownEvent: FakePointerEvent = new CustomEvent("pointerdown", { detail: data });
        this.#target.dispatchEvent(pointerDownEvent);
        break;
      case "pointerup":
        const pointerUpEvent: FakePointerEvent = new CustomEvent("pointerup", { detail: data });
        this.#target.dispatchEvent(pointerUpEvent);
        break;
    }
  }

  destroy() {
    this.#transformControls.dispose();
  }
}

import {
  Group,
  ObjectLoader,
  PMREMGenerator,
  PerspectiveCamera,
  Scene,
  WebGLRenderer,
  sRGBEncoding,
} from "three";
import { RoomEnvironment } from "three/examples/jsm/environments/RoomEnvironment";

import { Controls } from "../renderer/Controls";
import { FromRenderMessage, ToRenderMessage } from "../types";
import { disposeTree } from "../utils/disposeTree";

export type FromRenderPostMessage = (
  message: FromRenderMessage,
  transfer?: Transferable[] | undefined
) => void;

export class RenderWorker {
  #scene = new Scene();
  #objects = new Group();
  #renderer: WebGLRenderer | null = null;
  #camera: PerspectiveCamera | null = null;
  #animationFrameId: number | null = null;
  #canvas: HTMLCanvasElement | null = null;
  #canvasWidth = 0;
  #canvasHeight = 0;

  #controls: Controls | null = null;

  #postMessage: FromRenderPostMessage = (...args) => {
    this.postMessage(...args);
  };

  postMessage: FromRenderPostMessage = () => {};

  constructor(canvas?: HTMLCanvasElement) {
    this.#scene.add(this.#objects);
    if (canvas) this.#init(canvas);
  }

  onmessage(event: MessageEvent<ToRenderMessage>) {
    const { id, subject, data } = event.data;

    this.#controls?.onmessage(event);

    switch (subject) {
      case "add_object":
        this.#addObject(data.json);
        break;
      case "destroy":
        this.destroy();
        break;
      case "get_object":
        this.#getObject(data.uuid, id);
        break;
    }
  }

  async #addObject(json: any) {
    const loader = new ObjectLoader();
    const object = await loader.parseAsync(json);
    this.#objects.add(object);
  }

  async #getObject(uuid: string, id?: number) {
    const found = this.#objects.getObjectByProperty("uuid", uuid);
    const json = found?.toJSON();

    this.#postMessage({
      id,
      subject: "got_object",
      data: {
        json,
      },
    });
  }

  #init(canvas: HTMLCanvasElement) {
    this.#canvas = canvas;

    // Renderer
    this.#renderer = new WebGLRenderer({
      canvas,
      antialias: true,
      preserveDrawingBuffer: true,
    });
    this.#renderer.setPixelRatio(window.devicePixelRatio);
    this.#renderer.setSize(canvas.width, canvas.height);
    this.#renderer.outputEncoding = sRGBEncoding;
    this.#renderer.physicallyCorrectLights = true;

    // Environment
    const premGenerator = new PMREMGenerator(this.#renderer);
    premGenerator.compileEquirectangularShader();
    const environment = premGenerator.fromScene(new RoomEnvironment()).texture;
    this.#scene.environment = environment;

    // Camera
    this.#camera = new PerspectiveCamera(75, canvas.width / canvas.height);
    this.#camera.position.set(0, 0, 5);
    this.#camera.lookAt(0, 0, 0);

    // Controls
    this.#controls = new Controls(
      this.#camera,
      this.#renderer,
      this.#scene,
      this.#objects,
      this.#postMessage
    );

    // Start rendering
    this.#render();
  }

  #render() {
    this.#animationFrameId = requestAnimationFrame(() => this.#render());
    if (!this.#renderer || !this.#camera) return;

    this.#updateCanvasSize();
    this.#renderer.render(this.#scene, this.#camera);
  }

  #stop() {
    if (this.#animationFrameId !== null) cancelAnimationFrame(this.#animationFrameId);
  }

  #updateCanvasSize() {
    if (!this.#canvas || !this.#renderer || !this.#camera) return;

    const width = this.#canvas.width;
    const height = this.#canvas.height;

    if (width === this.#canvasWidth && height === this.#canvasHeight) return;

    this.#canvasWidth = width;
    this.#canvasHeight = height;

    this.#renderer.setSize(width, height);
    this.#camera.aspect = width / height;
    this.#camera.updateProjectionMatrix();
  }

  destroy() {
    // Stop rendering
    this.#stop();
    // Dispose rendering context
    this.#renderer?.dispose();
    // Dispose all objects
    disposeTree(this.#scene);
  }
}

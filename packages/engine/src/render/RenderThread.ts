import {
  AmbientLight,
  BoxGeometry,
  Fog,
  Mesh,
  MeshStandardMaterial,
  PCFSoftShadowMap,
  PerspectiveCamera,
  Scene,
  sRGBEncoding,
  WebGLRenderer,
} from "three";

import { isSceneMessage } from "../scene/messages";
import { PostMessage } from "../types";
import { FromRenderMessage, ToRenderMessage } from "./messages";
import { RenderScene } from "./RenderScene";

const CAMERA_NEAR = 0.1;
const CAMERA_FAR = 750;

export class RenderThread {
  #canvas: HTMLCanvasElement | OffscreenCanvas | null = null;
  postMessage: PostMessage<FromRenderMessage>;

  readonly renderScene = new RenderScene();
  readonly scene = new Scene();

  renderer: WebGLRenderer | null = null;
  size = { width: 0, height: 0 };
  pixelRatio = 1;
  camera = new PerspectiveCamera(75, 1, CAMERA_NEAR, CAMERA_FAR);

  constructor(postMessage: PostMessage<FromRenderMessage>) {
    this.postMessage = postMessage;

    this.scene.add(this.renderScene.root);
    this.scene.fog = new Fog(0xcfcfcf, CAMERA_FAR / 2, CAMERA_FAR);

    this.camera.position.set(0, 4, 0);
    this.camera.lookAt(0, 0, 0);

    const light = new AmbientLight(0xffffff, 0.5);
    this.scene.add(light);

    const cube = new Mesh(new BoxGeometry(1, 1, 1), new MeshStandardMaterial({ color: 0xff0000 }));
    this.scene.add(cube);

    this.render();
  }

  onmessage = (event: MessageEvent<ToRenderMessage>) => {
    if (isSceneMessage(event.data)) {
      this.renderScene.onmessage(event.data);
    }

    const { subject, data } = event.data;

    switch (subject) {
      case "set_canvas": {
        this.#canvas = data;
        this.init();
        break;
      }

      case "set_size": {
        this.size = data;
        this.camera.aspect = data.width / data.height;
        this.renderer?.setSize(data.width, data.height, false);
        break;
      }

      case "set_pixel_ratio": {
        this.pixelRatio = data;
        this.renderer?.setPixelRatio(data);
        break;
      }
    }
  };

  init() {
    if (!this.#canvas) return;

    this.renderer = new WebGLRenderer({
      canvas: this.#canvas,
      antialias: true,
      powerPreference: "high-performance",
      preserveDrawingBuffer: true,
    });

    this.renderer.setPixelRatio(this.pixelRatio);
    this.renderer.setSize(this.size.width, this.size.height, false);
    this.renderer.outputEncoding = sRGBEncoding;
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = PCFSoftShadowMap;
  }

  render() {
    requestAnimationFrame(() => this.render());
    if (!this.renderer) return;

    this.renderer.render(this.scene, this.camera);

    // Spin camera
    this.camera.position.x = 15 * Math.sin(Date.now() / 1000);
    this.camera.position.z = 15 * Math.cos(Date.now() / 1000);
    this.camera.lookAt(0, 0, 0);
  }
}

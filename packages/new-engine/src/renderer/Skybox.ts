import { CubeTextureLoader, Scene } from "three";

import { ToRenderMessage } from "../types";

export class Skybox {
  #scene: Scene;

  constructor(scene: Scene) {
    this.#scene = scene;
  }

  onmessage(event: MessageEvent<ToRenderMessage>) {
    const { subject, data } = event.data;

    switch (subject) {
      case "create_skybox":
        this.#createSkybox(data.path);
        break;
    }
  }

  #createSkybox(path: string) {
    // Skybox
    this.#scene.background = new CubeTextureLoader()
      .setPath(path)
      .load(["right.bmp", "left.bmp", "top.bmp", "bottom.bmp", "front.bmp", "back.bmp"]);
  }
}

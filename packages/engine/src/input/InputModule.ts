import { RenderModule } from "../render/RenderModule";
import { KeyboardInput } from "./KeyboardInput";

export class InputModule {
  keyboard: KeyboardInput;

  canvas: HTMLCanvasElement;
  render: RenderModule;

  inputArray: Int16Array | null = null;
  rotationArray: Int32Array | null = null;

  constructor(canvas: HTMLCanvasElement, render: RenderModule) {
    this.canvas = canvas;
    this.render = render;

    this.keyboard = new KeyboardInput(this);
  }

  destroy() {
    this.keyboard.destroy();
  }
}

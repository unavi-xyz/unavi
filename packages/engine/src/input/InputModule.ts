import { RenderModule } from "../render/RenderModule";
import { KeyboardInput } from "./KeyboardInput";

export class InputModule {
  keyboard: KeyboardInput;

  constructor(canvas: HTMLCanvasElement, render: RenderModule) {
    this.keyboard = new KeyboardInput(canvas, render);
  }

  destroy() {
    this.keyboard.destroy();
  }
}

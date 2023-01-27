import { Engine } from "../Engine";
import { KeyboardInput } from "./KeyboardInput";

export class InputModule {
  readonly engine: Engine;

  inputArray: Int16Array | null = null;
  rotationArray: Int32Array | null = null;

  keyboard: KeyboardInput;

  constructor(engine: Engine) {
    this.engine = engine;
    this.keyboard = new KeyboardInput(this);
  }

  destroy() {
    this.keyboard.destroy();
  }
}

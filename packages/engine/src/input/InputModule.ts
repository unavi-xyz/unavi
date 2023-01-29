import { Engine } from "../Engine";
import { KeyboardInput } from "./KeyboardInput";

export class InputModule {
  readonly engine: Engine;

  keyboard: KeyboardInput;

  constructor(engine: Engine) {
    this.engine = engine;
    this.keyboard = new KeyboardInput(this);
  }

  destroy() {
    this.keyboard.destroy();
  }
}

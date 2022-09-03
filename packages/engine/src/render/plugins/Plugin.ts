import { WorkerMessage } from "../../types";

export class Plugin {
  constructor() {}
  onmessage(event: MessageEvent<WorkerMessage>) {}
  animate(delta: number) {}
  destroy() {}
}

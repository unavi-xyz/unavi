import { RenderThread } from "./RenderThread";
import { TreeItem } from "./classes/TreeItem";

export class Engine {
  renderThread: RenderThread;
  tree = new TreeItem();

  constructor(canvas: HTMLCanvasElement) {
    this.tree.threeUUID = "root";
    this.renderThread = new RenderThread(canvas);
  }

  destroy() {
    this.renderThread.destroy();
  }
}

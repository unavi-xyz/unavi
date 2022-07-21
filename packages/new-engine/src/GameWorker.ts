import { RenderWorker } from "./RenderWorker";
import { GameLoadGltfMessage, MainMessage } from "./types";

export class GameWorker {
  private _worker = new Worker(new URL("./workers/Game.worker.ts", import.meta.url));

  private _renderWorker: RenderWorker;

  constructor(renderWorker: RenderWorker) {
    this._renderWorker = renderWorker;

    this._worker.onmessage = (event: MessageEvent<MainMessage>) => {
      const { type, data } = event.data;

      switch (type) {
        case "gltf":
          this._renderWorker.createGLTF(data);
          break;
      }
    };
  }

  public loadGltf(url: string) {
    const message: GameLoadGltfMessage = {
      type: "loadGltf",
      data: {
        url,
      },
    };
    this._worker.postMessage(message);
  }

  public update(delta: number) {}

  public destroy() {
    //terminate worker
    this._worker.terminate();
  }
}

import { ObservableV2 } from "lib0/observable";
import { applyUpdateV2, Doc } from "yjs";

import { useClientStore } from "../clientStore";

type Events = {
  update_incoming: (data: Uint8Array) => void;
  update_outgoing: (data: Uint8Array) => void;
};

export enum TxOrigin {
  Local,
  Remote,
  Internal,
}

export class YjsProvider extends ObservableV2<Events> {
  constructor(ydoc: Doc) {
    super();

    ydoc.on("updateV2", (data: Uint8Array, origin: TxOrigin) => {
      switch (origin) {
        case TxOrigin.Internal: {
          // Ignore updates from this provider
          break;
        }

        case TxOrigin.Local: {
          // Updates from this origin are outgoing updates
          this.emit("update_outgoing", [data]);
          break;
        }

        case TxOrigin.Remote: {
          // Updates from other origins are incoming updates
          this.emit("update_incoming", [data]);
          break;
        }
      }
    });

    this.on("update_incoming", (data) => {
      applyUpdateV2(ydoc, data, TxOrigin.Internal);
    });

    this.on("update_outgoing", (data) => {
      useClientStore.getState().sendEditorEvent({
        event: { oneofKind: "syncScene", syncScene: { data } },
      });
    });
  }
}

import { bind } from "valtio-yjs";
import { Doc } from "yjs";

import { syncedStore } from "../editor";
import { YjsProvider } from "./classes/YjsProvider";

export const ydoc = new Doc();
export const provider = new YjsProvider(ydoc);

const ymap = ydoc.getMap();
bind(syncedStore, ymap);

import { RenderWorker } from "./RenderWorker";

// @ts-ignore
const renderWorker = new RenderWorker(postMessage.bind(this));
onmessage = renderWorker.onmessage;

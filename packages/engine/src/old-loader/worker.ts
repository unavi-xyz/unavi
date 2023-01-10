import { LoaderWorker } from "./LoaderWorker";

// @ts-ignore
const loaderWorker = new LoaderWorker(postMessage.bind(this));
onmessage = loaderWorker.onmessage;

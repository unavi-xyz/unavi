import { LoaderWorker } from "../loader/LoaderWorker";

//@ts-ignore
const loaderWorker = new LoaderWorker(postMessage.bind(this));
onmessage = loaderWorker.onmessage;

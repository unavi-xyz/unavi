import { GameWorker } from "./GameWorker";

// @ts-ignore
const gameWorker = new GameWorker(postMessage.bind(this));
onmessage = gameWorker.onmessage;

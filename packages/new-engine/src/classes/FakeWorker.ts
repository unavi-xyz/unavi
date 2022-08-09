export class FakeWorker<To = any, From = any> {
  channel = new MessageChannel();
  workerPort = this.channel.port1;
  outsidePort = this.channel.port2;

  constructor(onMessage: (event: MessageEvent) => void) {
    this.workerPort.onmessage = onMessage;
    this.outsidePort.onmessage = this.#onmessage.bind(this);
  }

  postMessage(message: To) {
    this.outsidePort.postMessage(message);
  }

  #onmessage(event: MessageEvent<From>) {
    this.onmessage(event);
  }

  onmessage(event: MessageEvent<From>) {}
}

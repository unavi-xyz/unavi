export default class Worker {
  url: string;
  onmessage: (msg: any) => void;

  constructor(stringUrl: string) {
    this.url = stringUrl;
    this.onmessage = () => {};
  }

  postMessage(msg: any) {
    this.onmessage(msg);
  }
}

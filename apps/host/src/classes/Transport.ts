import { WebRtcTransport } from "mediasoup/node/lib/WebRtcTransport";

export class Transport {
  private _transport: WebRtcTransport | null = null;

  public set transport(transport: WebRtcTransport | null) {
    //close existing transport if it exists
    if (this._transport) {
      this._transport.close();
    }

    this._transport = transport;
  }

  public get transport() {
    return this._transport;
  }
}

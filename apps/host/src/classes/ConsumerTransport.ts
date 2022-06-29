import { Consumer, ConsumerOptions } from "mediasoup/node/lib/Consumer";

import { Transport } from "./Transport";

export class ConsumerTransport extends Transport {
  private _consumers: Consumer[] = [];

  constructor() {
    super();
  }

  public async consume(options: ConsumerOptions) {
    if (!this.transport) return;

    const newConsumer = await this.transport.consume(options);
    this._consumers.push(newConsumer);

    return newConsumer;
  }

  public async close() {
    for (const consumer of this._consumers) {
      consumer.close();
    }
    this._consumers = [];
    this.transport = null;
  }
}

import { Producer, ProducerOptions } from "mediasoup/node/lib/Producer";

import { Transport } from "./Transport";

export class ProducerTransport extends Transport {
  private _producer: Producer | null = null;

  constructor() {
    super();
  }

  public set producer(producer: Producer | null) {
    //close existing producer if it exists
    if (this._producer) {
      this._producer.close();
    }

    this._producer = producer;
  }

  public get producer() {
    return this._producer;
  }

  public async produce(options: ProducerOptions) {
    if (!this.transport) return;

    const newProducer = await this.transport.produce(options);
    this.producer = newProducer;

    return newProducer.id;
  }

  public async close() {
    this.producer = null;
    this.transport = null;
  }
}

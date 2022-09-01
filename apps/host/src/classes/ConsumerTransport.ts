import { Consumer, ConsumerOptions } from "mediasoup/node/lib/Consumer";
import {
  DataConsumer,
  DataConsumerOptions,
} from "mediasoup/node/lib/DataConsumer";

import { Transport } from "./Transport";

export class ConsumerTransport extends Transport {
  private _consumers = new Map<string, Consumer>();
  private _dataConsumers = new Map<string, DataConsumer>();

  constructor() {
    super();
  }

  public async close() {
    this._consumers.forEach((consumer) => consumer.close());
    this._dataConsumers.forEach((consumer) => consumer.close());

    this._consumers.clear();
    this._dataConsumers.clear();
    this.transport = null;
  }

  public async consume(options: ConsumerOptions) {
    if (!this.transport) return;

    //if consumer already exists, close it
    const oldConsumer = this._consumers.get(options.producerId);
    if (oldConsumer) {
      oldConsumer.close();
    }

    const newConsumer = await this.transport.consume(options);
    this._consumers.set(options.producerId, newConsumer);

    return newConsumer;
  }

  public async consumeData(options: DataConsumerOptions) {
    if (!this.transport) return;

    //if consumer already exists, close it
    const oldConsumer = this._dataConsumers.get(options.dataProducerId);
    if (oldConsumer) {
      oldConsumer.close();
    }

    const newConsumer = await this.transport.consumeData(options);
    this._dataConsumers.set(options.dataProducerId, newConsumer);

    return newConsumer;
  }
}

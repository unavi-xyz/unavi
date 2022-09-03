import { DataProducer } from "mediasoup/node/lib/DataProducer";
import { Producer, ProducerOptions } from "mediasoup/node/lib/Producer";
import { SctpStreamParameters } from "mediasoup/node/lib/SctpParameters";

import { Transport } from "./Transport";

export class ProducerTransport extends Transport {
  private _producer: Producer | null = null;
  private _dataProducer: DataProducer | null = null;

  constructor() {
    super();
  }

  public async close() {
    this.producer = null;
    this.dataProducer = null;
    this.transport = null;
  }

  //audio producer
  public set producer(producer: Producer | null) {
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

  //data producer
  public set dataProducer(dataProducer: DataProducer | null) {
    if (this._dataProducer) {
      this._dataProducer.close();
    }

    this._dataProducer = dataProducer;
  }

  public get dataProducer() {
    return this._dataProducer;
  }

  public async produceData(sctpStreamParameters: SctpStreamParameters) {
    if (!this.transport) return;

    const newDataProducer = await this.transport.produceData({
      sctpStreamParameters,
    });
    this._dataProducer = newDataProducer;

    return newDataProducer.id;
  }
}

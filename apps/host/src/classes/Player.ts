import { Consumer } from "mediasoup/node/lib/Consumer";
import { DataConsumer } from "mediasoup/node/lib/DataConsumer";
import { ProducerOptions } from "mediasoup/node/lib/Producer";
import { RtpCapabilities } from "mediasoup/node/lib/RtpParameters";
import { SctpStreamParameters } from "mediasoup/node/lib/SctpParameters";
import { nanoid } from "nanoid";

import { IChatMessage } from "@wired-xr/engine";

import { TypedSocket } from "../types";
import { ConsumerTransport } from "./ConsumerTransport";
import { GameManager } from "./GameManager";
import { ProducerTransport } from "./ProducerTransport";

export class Player {
  public handle: string | null = null;

  public readonly id: string;
  public readonly joinedSpaces = new Set<string>();

  public readonly producer = new ProducerTransport();
  public readonly consumer = new ConsumerTransport();

  private _consumers = new Map<string, Consumer[]>();
  private _dataConsumers = new Map<string, DataConsumer[]>();

  private _rtpCapabilities: RtpCapabilities | null = null;

  private _socket: TypedSocket;
  private _manager: GameManager;

  constructor(socket: TypedSocket, manager: GameManager) {
    this._socket = socket;
    this._manager = manager;

    this.id = socket.id;
  }

  //spaces
  public joinSpace(spaceId: string) {
    this._manager.getOrCreateSpace(spaceId).join(this);
    this.joinedSpaces.add(spaceId);
  }

  public leaveSpace(spaceId: string) {
    this._manager.getOrCreateSpace(spaceId).leave(this.id);
    this.joinedSpaces.delete(spaceId);

    //remove all consumers
    const consumers = this._consumers.get(spaceId);
    if (consumers) {
      for (const consumer of consumers) {
        consumer.close();
      }
    }
    this._consumers.delete(spaceId);

    //remove all data consumers
    const dataConsumers = this._dataConsumers.get(spaceId);
    if (dataConsumers) {
      for (const dataConsumer of dataConsumers) {
        dataConsumer.close();
      }
    }
    this._dataConsumers.delete(spaceId);
  }

  public disconnect() {
    //leave all spaces
    for (const spaceId of this.joinedSpaces) {
      this.leaveSpace(spaceId);
    }

    //close all transports
    this.producer.close();
    this.consumer.close();
  }

  public get identity() {
    //if no handle is set, return a guest name
    if (!this.handle) {
      return `Guest ${this.id.slice(0, 5)}`;
    }

    return this.handle;
  }

  public sendChatMessage(text: string) {
    const chatMessage: IChatMessage = {
      messageId: nanoid(),
      senderId: this.id,
      senderName: this.identity,
      text,
      timestamp: Date.now(),
    };

    //send to each space
    for (const spaceId of this.joinedSpaces) {
      const space = this._manager.getOrCreateSpace(spaceId);
      if (!space) continue;

      space.sendChatMessage(this.id, chatMessage);
    }
  }

  public recieveChatMessage(chatMessage: IChatMessage) {
    this._socket.emit("recieve_chat_message", chatMessage);
  }

  //audio
  set rtpCapabilities(rtpCapabilities: RtpCapabilities | null) {
    this._rtpCapabilities = rtpCapabilities;

    //broadcast to spaces
    //you will create a new consumer for each audio producer
    for (const spaceId of this.joinedSpaces) {
      this._manager.getOrCreateSpace(spaceId).registerNewAudioConsumer(this.id);
    }
  }

  get rtpCapabilities() {
    return this._rtpCapabilities;
  }

  public async produceAudio(options: ProducerOptions) {
    //create new producer
    const audioProducerId = await this.producer.produce(options);

    //broadcast to spaces
    //each player will create a new consumer for this producer
    for (const spaceId of this.joinedSpaces) {
      this._manager.getOrCreateSpace(spaceId).registerNewAudioProducer(this.id);
    }

    return audioProducerId;
  }

  public async consumeAudio(producerId: string, playerId: string, spaceId: string) {
    const rtpCapabilities = this._rtpCapabilities;
    if (!rtpCapabilities) return;

    const canConsume = this._manager.router.canConsume({
      producerId,
      rtpCapabilities,
    });
    if (!canConsume) return;

    const consumer = await this.consumer.consume({
      producerId,
      rtpCapabilities,
    });
    if (!consumer) return;

    //add to list
    const space = this._consumers.get(spaceId);
    if (!space) {
      this._consumers.set(spaceId, [consumer]);
    } else {
      space.push(consumer);
    }

    //send consumer to the client
    this._socket.emit("new_audio_consumer", {
      playerId,
      producerId,
      id: consumer.id,
      kind: consumer.kind,
      rtpParameters: consumer.rtpParameters,
      type: consumer.type,
      producerPaused: consumer.producerPaused,
    });
  }

  //data
  public async produceData(params: SctpStreamParameters) {
    //create new producer
    const dataProducerId = await this.producer.produceData(params);

    //broadcast to spaces
    //each player will create a new consumer for this producer
    for (const spaceId of this.joinedSpaces) {
      this._manager.getOrCreateSpace(spaceId).registerNewDataProducer(this.id);
    }

    return dataProducerId;
  }

  public async consumeData(dataProducerId: string, playerId: string, spaceId: string) {
    const dataConsumer = await this.consumer.consumeData({
      dataProducerId,
    });
    if (!dataConsumer) return;

    //add to list
    const space = this._dataConsumers.get(spaceId);
    if (!space) {
      this._dataConsumers.set(spaceId, [dataConsumer]);
    } else {
      space.push(dataConsumer);
    }

    //send consumer to the client
    this._socket.emit("new_data_consumer", {
      playerId,
      dataProducerId,
      id: dataConsumer.id,
      sctpStreamParameters: dataConsumer.sctpStreamParameters,
    });
  }
}

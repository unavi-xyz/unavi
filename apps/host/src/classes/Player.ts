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
    this._manager.getSpace(spaceId).join(this);
    this.joinedSpaces.add(spaceId);
  }

  public leaveSpace(spaceId: string) {
    this._manager.getSpace(spaceId).leave(this.id);
    this.joinedSpaces.delete(spaceId);
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
      const space = this._manager.getSpace(spaceId);
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
      this._manager.getSpace(spaceId).registerNewAudioConsumer(this.id);
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
      this._manager.getSpace(spaceId).registerNewAudioProducer(this.id);
    }

    return audioProducerId;
  }

  public async consumeAudio(producerId: string) {
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

    //send consumer to the client
    this._socket.emit("new_audio_consumer", {
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
      this._manager.getSpace(spaceId).registerNewDataProducer(this.id);
    }

    return dataProducerId;
  }

  public async consumeData(dataProducerId: string) {
    const dataConsumer = await this.consumer.consumeData({
      dataProducerId,
    });
    if (!dataConsumer) return;

    //send consumer to the client
    this._socket.emit("new_data_consumer", {
      dataProducerId,
      id: dataConsumer.id,
      sctpStreamParameters: dataConsumer.sctpStreamParameters,
    });
  }
}

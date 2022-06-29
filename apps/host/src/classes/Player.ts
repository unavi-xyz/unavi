import { ProducerOptions } from "mediasoup/node/lib/Producer";
import { RtpCapabilities } from "mediasoup/node/lib/RtpParameters";
import { Socket } from "socket.io";

import { ConsumerTransport } from "./ConsumerTransport";
import { GameManager } from "./GameManager";
import { ProducerTransport } from "./ProducerTransport";

export class Player {
  public readonly id: string;
  public readonly joinedSpaces = new Set<string>();

  public readonly audioProducer = new ProducerTransport();
  public readonly audioConsumer = new ConsumerTransport();

  private _rtpCapabilities: RtpCapabilities | null = null;

  private _socket: Socket;
  private _manager: GameManager;

  constructor(socket: Socket, manager: GameManager) {
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
    this.audioProducer.close();
    this.audioConsumer.close();
  }

  //audio
  public async produceAudio(options: ProducerOptions) {
    //create new producer
    const newId = await this.audioProducer.produce(options);

    //broadcast to spaces
    //each player will create a new consumer for this producer
    for (const spaceId of this.joinedSpaces) {
      this._manager.getSpace(spaceId).registerNewAudioProducer(this.id);
    }

    return newId;
  }

  set rtpCapabilities(rtpCapabilities: RtpCapabilities | null) {
    this._rtpCapabilities = rtpCapabilities;

    //broadcast to spaces
    //you will create a new consumer for each producer
    for (const spaceId of this.joinedSpaces) {
      this._manager.getSpace(spaceId).registerNewAudioConsumer(this.id);
    }
  }

  get rtpCapabilities() {
    return this._rtpCapabilities;
  }

  public async consumeAudio(producerId: string) {
    const rtpCapabilities = this._rtpCapabilities;
    if (!rtpCapabilities) return;

    const canConsume = this._manager.router.canConsume({
      producerId,
      rtpCapabilities,
    });
    if (!canConsume) return;

    const consumer = await this.audioConsumer.consume({
      producerId,
      rtpCapabilities,
    });
    if (!consumer) return;

    //send consumer to the client
    this._socket.emit("new_consumer", {
      producerId,
      id: consumer.id,
      kind: consumer.kind,
      rtpParameters: consumer.rtpParameters,
      type: consumer.type,
      producerPaused: consumer.producerPaused,
    });
  }
}

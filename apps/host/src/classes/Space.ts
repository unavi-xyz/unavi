import { GameManager } from "./GameManager";
import { Player } from "./Player";

export class Space {
  public readonly id: string;

  private _audioProducers = new Set<string>();
  private _dataProducers = new Set<string>();

  private _manager: GameManager;
  private _players = new Map<string, Player>();

  constructor(id: string, manager: GameManager) {
    this.id = id;
    this._manager = manager;
  }

  public get playercount() {
    return this._players.size;
  }

  public join(player: Player) {
    this._players.set(player.id, player);
  }

  public leave(playerId: string) {
    //remove player from space
    this._players.delete(playerId);

    //remove player from producers
    this._audioProducers.delete(playerId);

    //if no players left, delete space
    if (this.playercount === 0) {
      this._manager.deleteSpace(this.id);
    }
  }

  //audio
  public registerNewAudioProducer(playerId: string) {
    const player = this._players.get(playerId);
    if (!player) return;

    const producer = player.producer.producer;
    if (!producer) return;

    //add to producers
    this._audioProducers.add(playerId);

    //tell all players in the space to consume this producer
    for (const player of this._players.values()) {
      if (player.id === playerId) return;
      player.consumeAudio(producer.id);
    }
  }

  public registerNewAudioConsumer(playerId: string) {
    const player = this._players.get(playerId);
    if (!player) return;

    //create a consumer for each producer in the space
    for (const producerPlayerId of this._audioProducers.values()) {
      if (producerPlayerId === playerId) return;

      const producerPlayer = this._players.get(producerPlayerId);
      if (!producerPlayer) {
        //this should never happen
        this._audioProducers.delete(producerPlayerId);
        return;
      }

      const producer = producerPlayer.producer.producer;
      if (!producer) {
        //this should never happen
        this._audioProducers.delete(producerPlayerId);
        return;
      }

      player.consumeAudio(producer.id);
    }
  }

  //data
  public registerNewDataProducer(playerId: string) {
    const player = this._players.get(playerId);
    if (!player) return;

    const dataProducer = player.producer.dataProducer;
    if (!dataProducer) return;

    //start consuming data
    this.registerNewDataConsumer(player.id);

    //add to producers
    this._dataProducers.add(playerId);

    //tell all players in the space to consume this producer
    for (const player of this._players.values()) {
      if (player.id === playerId) return;
      player.consumeData(dataProducer.id);
    }
  }

  public registerNewDataConsumer(playerId: string) {
    const player = this._players.get(playerId);
    if (!player) return;

    //create a consumer for each producer in the space
    for (const producerPlayerId of this._dataProducers.values()) {
      if (producerPlayerId === playerId) return;

      const producerPlayer = this._players.get(producerPlayerId);
      if (!producerPlayer) {
        //this should never happen
        this._dataProducers.delete(producerPlayerId);
        return;
      }

      const dataProducer = producerPlayer.producer.dataProducer;
      if (!dataProducer) {
        //this should never happen
        this._dataProducers.delete(producerPlayerId);
        return;
      }

      player.consumeData(dataProducer.id);
    }
  }
}

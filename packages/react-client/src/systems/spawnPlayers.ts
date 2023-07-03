import { Time } from "lattice-engine/core";
import {
  GlobalTransform,
  Parent,
  SceneStruct,
  Transform,
} from "lattice-engine/scene";
import { Vrm } from "lattice-engine/vrm";
import { Commands, dropStruct, Entity, EventReader, Query, Res } from "thyseus";

import { NetworkTransform, OtherPlayer } from "../components";
import { PlayerJoin, PlayerLeave } from "../events";
import { useClientStore } from "../store";

export function spawnPlayers(
  commands: Commands,
  time: Res<Time>,
  sceneStruct: Res<SceneStruct>,
  playerJoin: EventReader<PlayerJoin>,
  playerLeave: EventReader<PlayerLeave>,
  players: Query<[Entity, OtherPlayer]>
) {
  for (const event of playerJoin) {
    const otherPlayer = new OtherPlayer();
    otherPlayer.id = event.playerId;

    const vrm = new Vrm();
    vrm.uri = useClientStore.getState().defaultAvatar;

    const parent = new Parent();
    parent.id = sceneStruct.activeScene;

    const networkTransform = new NetworkTransform();
    networkTransform.lastUpdate = time.mainTime;
    networkTransform.transform.rotation.set(0, 0, 0, 1);
    networkTransform.transform.scale.set(1, 1, 1);

    commands
      .spawn(true)
      .addType(Transform)
      .addType(GlobalTransform)
      .add(networkTransform)
      .add(parent)
      .add(otherPlayer)
      .add(vrm);

    dropStruct(otherPlayer);
    dropStruct(vrm);
    dropStruct(parent);
    dropStruct(networkTransform);
  }

  for (const event of playerLeave) {
    for (const [entity, player] of players) {
      if (player.id === event.playerId) {
        commands.despawn(entity);
        useClientStore.getState().locations.delete(player.id);
      }
    }
  }

  playerJoin.clear();
  playerLeave.clear();
}

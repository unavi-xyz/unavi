import { Time } from "houseki/core";
import { Velocity } from "houseki/physics";
import { PlayerAvatar, PlayerBody } from "houseki/player";
import { GlobalTransform, Parent, SceneStruct, Transform } from "houseki/scene";
import { Vrm } from "houseki/vrm";
import { Commands, Entity, EventReader, Query, Res } from "thyseus";

import { useClientStore } from "../clientStore";
import { NetworkTransform, OtherPlayer, PrevTranslation } from "../components";
import { PlayerJoin, PlayerLeave } from "../events";

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

    const parent = new Parent();
    parent.id = sceneStruct.activeScene;

    const networkTransform = new NetworkTransform();
    networkTransform.lastUpdate = time.mainTime;
    networkTransform.transform.rotation.set(0, 0, 0, 1);
    networkTransform.transform.scale.set(1, 1, 1);

    const bodyId = commands
      .spawn(true)
      .addType(Transform)
      .addType(GlobalTransform)
      .add(networkTransform)
      .add(parent)
      .addType(PrevTranslation)
      .addType(Velocity)
      .addType(PlayerBody)
      .add(otherPlayer).id;

    const playerAvatar = new PlayerAvatar();
    playerAvatar.idleAnimation = "/models/Idle.fbx";
    playerAvatar.jumpAnimation = "/models/Falling.fbx";
    playerAvatar.leftWalkAnimation = "/models/LeftWalk.fbx";
    playerAvatar.rightWalkAnimation = "/models/RightWalk.fbx";
    playerAvatar.sprintAnimation = "/models/Sprint.fbx";
    playerAvatar.walkAnimation = "/models/Walk.fbx";

    const vrm = new Vrm();
    vrm.uri = useClientStore.getState().defaultAvatar;

    commands
      .spawn(true)
      .add(parent.setId(bodyId))
      .addType(Transform)
      .addType(GlobalTransform)
      .add(playerAvatar)
      .add(vrm);
  }

  for (const event of playerLeave) {
    for (const [entity, player] of players) {
      if (player.id === event.playerId) {
        commands.despawn(entity);
      }
    }
  }

  playerJoin.clear();
  playerLeave.clear();
}

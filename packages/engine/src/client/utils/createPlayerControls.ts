import { Warehouse } from "lattice-engine/core";
import {
  CapsuleCollider,
  CharacterController,
  KinematicBody,
  Raycast,
  TargetTransform,
  Velocity,
} from "lattice-engine/physics";
import {
  PlayerAvatar,
  PlayerBody,
  PlayerCamera,
  PlayerCameraMode,
  PlayerCameraView,
  TargetRotation,
  TargetTranslation,
} from "lattice-engine/player";
import {
  GlobalTransform,
  Parent,
  PerspectiveCamera,
  SceneStruct,
  Transform,
} from "lattice-engine/scene";
import { Vrm } from "lattice-engine/vrm";
import { Commands } from "thyseus";

const PLAYER_HEIGHT = 1.6;
const PLAYER_WIDTH = 0.4;

export function createPlayerControls(
  warehouse: Warehouse,
  spawn: [number, number, number],
  rootId: bigint,
  commands: Commands,
  sceneStruct: SceneStruct
) {
  const parent = new Parent();
  const transform = new Transform();

  const player = new PlayerBody();
  player.spawnPoint.fromArray(spawn);

  const targetTransform = new TargetTransform();
  const targetRotation = new TargetRotation(0, 0, 0, 1);
  const capsuleCollider = new CapsuleCollider(
    PLAYER_WIDTH,
    PLAYER_HEIGHT - PLAYER_WIDTH * 2
  );

  const bodyId = commands
    .spawn(true)
    .add(parent.setId(rootId))
    .add(transform.set(spawn))
    .add(targetTransform.set(spawn))
    .add(targetRotation)
    .addType(GlobalTransform)
    .addType(Velocity)
    .add(capsuleCollider)
    .addType(KinematicBody)
    .addType(CharacterController)
    .add(player).id;

  const playerAvatar = new PlayerAvatar();
  playerAvatar.idleAnimation.write("/models/Idle.fbx", warehouse);
  playerAvatar.jumpAnimation.write("/models/Falling.fbx", warehouse);
  playerAvatar.leftWalkAnimation.write("/models/LeftWalk.fbx", warehouse);
  playerAvatar.rightWalkAnimation.write("/models/RightWalk.fbx", warehouse);
  playerAvatar.sprintAnimation.write("/models/Sprint.fbx", warehouse);
  playerAvatar.walkAnimation.write("/models/Walk.fbx", warehouse);

  const vrm = new Vrm();
  vrm.uri.write("/models/Robot.vrm", warehouse);
  vrm.setupFirstPerson = true;

  commands
    .spawn(true)
    .add(transform.set([0, -PLAYER_HEIGHT / 2, 0]))
    .addType(GlobalTransform)
    .add(parent.setId(bodyId))
    .add(vrm)
    .add(playerAvatar);

  const playerCamera = new PlayerCamera(
    PlayerCameraMode.Both,
    PlayerCameraView.ThirdPerson
  );
  playerCamera.bodyId = bodyId;

  const cameraId = commands
    .spawn(true)
    .addType(Transform)
    .addType(GlobalTransform)
    .addType(TargetTranslation)
    .add(targetRotation)
    .addType(PerspectiveCamera)
    .add(playerCamera)
    .addType(Raycast).id;

  sceneStruct.activeCamera = cameraId;

  return cameraId;
}

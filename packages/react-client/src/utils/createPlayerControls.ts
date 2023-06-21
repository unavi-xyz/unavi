import { InputStruct } from "lattice-engine/input";
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
  TargetPosition,
  TargetRotation,
} from "lattice-engine/player";
import {
  GlobalTransform,
  Parent,
  PerspectiveCamera,
  SceneStruct,
  Transform,
} from "lattice-engine/scene";
import { Vrm } from "lattice-engine/vrm";
import { Commands, dropStruct, EntityCommands } from "thyseus";

const PLAYER_HEIGHT = 1.6;
const PLAYER_WIDTH = 0.4;

export function createPlayerControls(
  root: EntityCommands,
  commands: Commands,
  sceneStruct: SceneStruct,
  inputStruct: InputStruct,
  spawn: [number, number, number] = [0, 2, 0]
) {
  const parent = new Parent();
  const transform = new Transform();

  const player = new PlayerBody();
  player.spawnPoint.fromArray(spawn);

  const targetTransform = new TargetTransform();
  const capsuleCollider = new CapsuleCollider(
    PLAYER_WIDTH,
    PLAYER_HEIGHT - PLAYER_WIDTH * 2
  );

  const body = commands
    .spawn()
    .add(parent.setEntity(root))
    .add(transform.set(spawn))
    .add(targetTransform.set(spawn))
    .addType(GlobalTransform)
    .addType(Velocity)
    .add(capsuleCollider)
    .addType(KinematicBody)
    .addType(CharacterController)
    .add(player);

  dropStruct(player);
  dropStruct(targetTransform);
  dropStruct(capsuleCollider);

  const playerAvatar = new PlayerAvatar();
  playerAvatar.idleAnimation = "/models/Idle.fbx";
  playerAvatar.jumpAnimation = "/models/Falling.fbx";
  playerAvatar.leftWalkAnimation = "/models/LeftWalk.fbx";
  playerAvatar.rightWalkAnimation = "/models/RightWalk.fbx";
  playerAvatar.sprintAnimation = "/models/Sprint.fbx";
  playerAvatar.walkAnimation = "/models/Walk.fbx";

  const targetRotation = new TargetRotation();
  const vrm = new Vrm("/models/Robot.vrm", true);

  commands
    .spawn()
    .add(transform.set([0, -PLAYER_HEIGHT / 2, 0]))
    .addType(GlobalTransform)
    .add(targetRotation.set(0, 0, 0, 1))
    .add(parent.setEntity(body))
    .add(vrm)
    .add(playerAvatar);

  dropStruct(playerAvatar);
  dropStruct(vrm);

  const playerCamera = new PlayerCamera(
    PlayerCameraMode.Both,
    PlayerCameraView.ThirdPerson
  );

  const camera = commands
    .spawn()
    .addType(Transform)
    .addType(GlobalTransform)
    .addType(TargetPosition)
    .add(targetRotation.set(0, 0, 0, 1))
    .add(parent.setEntity(body))
    .addType(PerspectiveCamera)
    .add(playerCamera)
    .addType(Raycast);

  dropStruct(parent);
  dropStruct(transform);
  dropStruct(playerCamera);
  dropStruct(targetRotation);

  sceneStruct.activeCamera = camera.id;
  inputStruct.enablePointerLock = true;
}

import {
  BoxGeometry,
  Mesh,
  MeshStandardMaterial,
  Quaternion,
  Vector3,
} from "three";

const LERP_FACTOR = 0.00001;

export class OtherPlayer {
  readonly playerId: string;
  readonly mesh: Mesh;

  #targetPosition = new Vector3();
  #targetRotation = new Quaternion();

  constructor(playerId: string) {
    this.playerId = playerId;

    const geometry = new BoxGeometry(1, 1, 1);
    const material = new MeshStandardMaterial({ color: 0xff3333 });
    this.mesh = new Mesh(geometry, material);
  }

  setLocation(
    location: [number, number, number, number, number, number, number]
  ) {
    this.#targetPosition.set(location[0], location[1], location[2]);
    this.#targetRotation.set(
      location[3],
      location[4],
      location[5],
      location[6]
    );
  }

  animate(delta: number) {
    const K = 1 - Math.pow(LERP_FACTOR, delta);

    this.mesh.position.lerp(this.#targetPosition, K);
    this.mesh.quaternion.slerp(this.#targetRotation, K);
  }

  destroy() {
    this.mesh.removeFromParent();
    this.mesh.geometry.dispose();
  }
}

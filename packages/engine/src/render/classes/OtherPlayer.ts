import { BoxGeometry, Mesh, MeshBasicMaterial } from "three";

import { Quad, Triplet } from "../../types";

export class OtherPlayer {
  readonly playerId: string;
  readonly mesh: Mesh;

  #interpolatedPosition: Triplet = [0, 0, 0];
  #interpolatedRotation: Quad = [0, 0, 0, 1];

  #targetPosition: Triplet = [0, 0, 0];
  #targetRotation: Quad = [0, 0, 0, 1];

  constructor(playerId: string) {
    this.playerId = playerId;

    const geometry = new BoxGeometry(1, 1, 1);
    const material = new MeshBasicMaterial({ color: 0x00ff00 });
    this.mesh = new Mesh(geometry, material);

    this.mesh.position.set(0, 0, 0);
  }

  setLocation(
    location: [number, number, number, number, number, number, number]
  ) {
    this.#targetPosition[0] = location[0];
    this.#targetPosition[1] = location[1];
    this.#targetPosition[2] = location[2];

    this.#targetRotation[0] = location[3];
    this.#targetRotation[1] = location[4];
    this.#targetRotation[2] = location[5];
    this.#targetRotation[3] = location[6];
  }

  animate() {
    this.#interpolatedPosition[0] +=
      (this.#targetPosition[0] - this.#interpolatedPosition[0]) / 10;
    this.#interpolatedPosition[1] +=
      (this.#targetPosition[1] - this.#interpolatedPosition[1]) / 10;
    this.#interpolatedPosition[2] +=
      (this.#targetPosition[2] - this.#interpolatedPosition[2]) / 10;

    this.#interpolatedRotation[0] +=
      (this.#targetRotation[0] - this.#interpolatedRotation[0]) / 10;
    this.#interpolatedRotation[1] +=
      (this.#targetRotation[1] - this.#interpolatedRotation[1]) / 10;
    this.#interpolatedRotation[2] +=
      (this.#targetRotation[2] - this.#interpolatedRotation[2]) / 10;
    this.#interpolatedRotation[3] +=
      (this.#targetRotation[3] - this.#interpolatedRotation[3]) / 10;

    this.mesh.position.set(
      this.#interpolatedPosition[0],
      this.#interpolatedPosition[1],
      this.#interpolatedPosition[2]
    );

    this.mesh.quaternion.set(
      this.#interpolatedRotation[0],
      this.#interpolatedRotation[1],
      this.#interpolatedRotation[2],
      this.#interpolatedRotation[3]
    );
  }

  destroy() {
    this.mesh.removeFromParent();
    this.mesh.geometry.dispose();
  }
}

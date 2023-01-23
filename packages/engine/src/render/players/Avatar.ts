import { VRM, VRMLoaderPlugin, VRMUtils } from "@pixiv/three-vrm";
import { Group, Mesh, Quaternion } from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader";

export class Avatar {
  group = new Group();
  vrm: VRM | null = null;

  #mode: "first-person" | "third-person" = "third-person";
  #quat = new Quaternion();
  #quatb = new Quaternion();

  targetRotation = new Quaternion();

  constructor(uri: string) {
    const loader = new GLTFLoader();
    loader.setCrossOrigin("anonymous");
    loader.register((parser) => new VRMLoaderPlugin(parser));

    loader.load(uri, (gltf) => {
      const vrm = gltf.userData.vrm as VRM;
      vrm.scene.rotateY(Math.PI);
      this.vrm = vrm;

      VRMUtils.removeUnnecessaryVertices(vrm.scene);
      VRMUtils.removeUnnecessaryJoints(vrm.scene);
      VRMUtils.rotateVRM0(vrm);

      vrm.scene.traverse((object) => {
        object.frustumCulled = false;
        if (object instanceof Mesh) object.castShadow = true;
      });

      if (this.#mode === "first-person") vrm.firstPerson?.setup();

      this.group.add(vrm.scene);
    });
  }

  get mode() {
    return this.#mode;
  }

  set mode(mode: "first-person" | "third-person") {
    this.#mode = mode;
    if (mode === "first-person") this.vrm?.firstPerson?.setup();
  }

  update(delta: number) {
    if (!this.vrm) return;

    this.vrm.update(delta);

    // Rotate body around Y axis
    this.group.quaternion.copy(this.targetRotation);
    this.group.quaternion.x = 0;
    this.group.quaternion.z = 0;
    this.group.quaternion.normalize();

    // Rotate head up and down
    const head = this.vrm.humanoid.getNormalizedBoneNode("head");
    if (head) {
      // Get relative rotation
      const relativeRotation = this.#quat
        .copy(this.targetRotation)
        .premultiply(this.#quatb.copy(this.group.quaternion).invert());

      // Don't rotate Y axis
      relativeRotation.y = 0;
      relativeRotation.normalize();

      if (this.vrm.meta.metaVersion === "1") {
        // If vrm 1.0, rotate axis
        const rotated = this.#quatb.copy(relativeRotation);
        rotated.x = relativeRotation.z;
        rotated.z = -relativeRotation.x;
        head.quaternion.copy(rotated);
      } else {
        head.quaternion.copy(relativeRotation);
      }
    }
  }

  dispose() {
    this.group.removeFromParent();
  }
}

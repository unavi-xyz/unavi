import { VRM, VRMLoaderPlugin, VRMUtils } from "@pixiv/three-vrm";
import { Group, Mesh } from "three";
import { CSM } from "three/examples/jsm/csm/CSM";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader";

export class Avatar {
  group = new Group();
  vrm: VRM | null = null;

  #csm: CSM | null = null;
  #mode: "first-person" | "third-person" = "third-person";

  constructor(uri: string) {
    const loader = new GLTFLoader();
    loader.setCrossOrigin("anonymous");
    loader.register((parser) => new VRMLoaderPlugin(parser));

    loader.load(uri, (gltf) => {
      const vrm = gltf.userData.vrm as VRM;
      this.vrm = vrm;

      VRMUtils.removeUnnecessaryVertices(vrm.scene);
      VRMUtils.removeUnnecessaryJoints(vrm.scene);
      VRMUtils.rotateVRM0(vrm);

      vrm.scene.traverse((object) => {
        if (object instanceof Mesh) object.castShadow = true;
      });

      this.#applyCSM();

      if (this.#mode === "first-person") this.#setupFirstPerson();

      this.group.add(vrm.scene);
    });
  }

  get mode() {
    return this.#mode;
  }

  set mode(mode: "first-person" | "third-person") {
    this.#mode = mode;
    if (mode === "first-person") this.#setupFirstPerson();
  }

  get csm() {
    return this.#csm;
  }

  set csm(csm: CSM | null) {
    this.#csm = csm;
    this.#applyCSM();
  }

  #setupFirstPerson() {
    if (!this.vrm?.firstPerson) return;

    // Force mesh annotations to be auto when in first-person mode
    // This prevents avatars from covering the camera
    this.vrm.firstPerson.meshAnnotations.forEach((annotation) => {
      if (annotation.type === "both") annotation.type = "auto";
    });

    setTimeout(() => this.vrm?.firstPerson?.setup(), 1000);
  }

  #applyCSM() {
    // this.vrm?.scene.traverse((object) => {
    //   if (object instanceof Mesh) {
    //     const materials: Material[] = Array.isArray(object.material)
    //       ? object.material
    //       : [object.material];
    //     materials.forEach((material) => {
    //       this.#csm?.setupMaterial(material);
    //     });
    //   }
    // });
  }

  update(delta: number) {
    this.vrm?.update(delta);
  }

  dispose() {
    this.group.removeFromParent();
  }
}

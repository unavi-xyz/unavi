// See: https://threejs.org/docs/#manual/en/introduction/How-to-dispose-of-objects

import { BufferGeometry, IUniform, Material, Object3D, Skeleton, Texture } from "three";

function disposeMaterial(material: Material): void {
  Object.values(material).forEach((value) => {
    if (value?.isTexture) {
      const texture = value as Texture;
      texture.dispose();
    }
  });

  if ((material as any).isShaderMaterial) {
    const uniforms: { [uniform: string]: IUniform<any> } = (material as any).uniforms;
    if (uniforms) {
      Object.values(uniforms).forEach((uniform) => {
        const value = uniform.value;
        if (value?.isTexture) {
          const texture = value as Texture;
          texture.dispose();
        }
      });
    }
  }

  material.dispose();
}

function dispose(object3D: Object3D): void {
  const geometry: BufferGeometry | undefined = (object3D as any).geometry;
  if (geometry) {
    geometry.dispose();
  }

  const skeleton: Skeleton | undefined = (object3D as any).skeleton;
  if (skeleton) {
    skeleton.dispose();
  }

  const material: Material | Material[] | undefined = (object3D as any).material;
  if (material) {
    if (Array.isArray(material)) {
      material.forEach((material: Material) => disposeMaterial(material));
    } else if (material) {
      disposeMaterial(material);
    }
  }
}

export function deepDispose(object3D: Object3D): void {
  object3D.traverse(dispose);
}

import { Mesh, MeshBasicMaterial, Scene, SphereGeometry, Vector3 } from "three";

export function visualizePoint(scene: Scene, point: Vector3, color = 0xff4400) {
  const geometry = new SphereGeometry(0.015, 16, 16);
  const material = new MeshBasicMaterial({
    color,
  });
  const sphere = new Mesh(geometry, material);
  sphere.position.copy(point);
  scene.add(sphere);
  return sphere;
}

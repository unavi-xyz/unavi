import { Camera, Mesh, Object3D, WebGLRenderer } from "three";

const MAX_COMPILE_TIME = 2; // Spend at most X ms compiling materials each frame

/*
 * Adds objects to the scene over time to avoid lag spikes.
 */
export class ObjectQueue {
  queue = new Array<{ object: Object3D; parent: Object3D }>();

  #renderer: WebGLRenderer;
  #camera: Camera;

  constructor(renderer: WebGLRenderer, camera: Camera) {
    this.#renderer = renderer;
    this.#camera = camera;
  }

  add(object: Object3D, parent: Object3D, useFrustum = false) {
    // Add to parent and make invisible
    parent.add(object);

    if (useFrustum) {
      // Hack to force objects to render for a frame
      object.traverse((child) => {
        child.frustumCulled = false;
        child.onAfterRender = () => {
          child.frustumCulled = true;
          child.onAfterRender = () => {};
        };
      });
    } else {
      // Make invisible
      object.visible = false;

      // Add children to queue first
      object.children.forEach((child) => this.add(child, object));

      // Add to queue
      this.queue.push({ object, parent });
    }
  }

  update() {
    if (this.queue.length === 0) return;

    let compileTime = 0;

    while (this.queue.length > 0 && compileTime < MAX_COMPILE_TIME) {
      // Take the next object from the queue
      const item = this.queue.shift();
      if (item) {
        const start = performance.now();

        // Make visible
        item.object.visible = true;

        // Compile materials
        if (item.object instanceof Mesh) this.#renderer.compile(item.object, this.#camera);

        // Add to compile time
        const difference = performance.now() - start;
        compileTime += difference;
      }
    }
  }
}

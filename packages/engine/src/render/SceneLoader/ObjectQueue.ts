import { Camera, Group, Object3D, WebGLRenderer } from "three";

const MAX_COMPILE_TIME = 2; // Spend at most X ms compiling materials each frame

/*
 * Adds objects to the scene over time to avoid lag spikes.
 */
export class ObjectQueue {
  queue = new Array<{ object: Object3D; parent: Object3D }>();

  waitingGroup = new Group();
  waitingQueue = new Array<{ object: Object3D; parent: Object3D }>();

  #renderer: WebGLRenderer;
  #camera: Camera;

  constructor(renderer: WebGLRenderer, camera: Camera, scene: Object3D) {
    this.#renderer = renderer;
    this.#camera = camera;

    this.waitingGroup.scale.set(0, 0, 0);
    scene.add(this.waitingGroup);
  }

  add(object: Object3D, parent: Object3D) {
    // Add children to the queue
    // TODO load each child individually
    // object.children.forEach((child) => this.add(child, object));

    this.queue.push({ object, parent });
  }

  update() {
    if (this.queue.length === 0) return;

    let compileTime = 0;

    while (this.queue.length > 0 && compileTime < MAX_COMPILE_TIME) {
      // Take the next object from the queue
      const item = this.queue.shift();
      if (item) {
        const start = performance.now();

        // Add it either to either the parent or the waiting group
        let isParentInScene = false;
        item.parent.traverseAncestors((a) => {
          if (a.type === "Scene") isParentInScene = true;
        });

        if (isParentInScene) {
          // Add to parent
          item.parent.add(item.object);
        } else {
          // Add to waiting group
          item.object.removeFromParent();
          this.waitingGroup.add(item.object);
          this.waitingQueue.push(item);
        }

        // Add any waiting children
        this.waitingQueue.forEach((e, i) => {
          if (e.parent === item.object) {
            item.object.add(e.object);
            this.waitingQueue.splice(i, 1);
          }
        });

        // Compile materials
        this.#renderer.compile(item.object, this.#camera);

        // Add to compile time
        const difference = performance.now() - start;
        compileTime += difference;
      }
    }
  }
}

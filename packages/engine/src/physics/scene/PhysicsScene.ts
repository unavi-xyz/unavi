import { World } from "@dimforge/rapier3d";

import { Scene } from "../../scene/Scene";
import { ToPhysicsMessage } from "../messages";
import { NodeBuilder } from "./NodeBuilder";

/**
 * Instance of a {@link Scene} for the physics thread.
 * This class is responsible for creating and disposing physics objects.
 */
export class PhysicsScene extends Scene {
  world: World;

  builders = {
    node: new NodeBuilder(this),
  };

  constructor(world: World) {
    super();
    this.world = world;
  }

  onmessage = ({ subject, data }: ToPhysicsMessage) => {
    switch (subject) {
      case "create_buffer": {
        this.buffer.create(data.json, data.id);
        break;
      }

      case "dispose_buffer": {
        const buffer = this.buffer.store.get(data);
        if (!buffer) throw new Error("Buffer not found");
        buffer.dispose();
        break;
      }

      case "create_accessor": {
        this.accessor.create(data.json, data.id);
        break;
      }

      case "dispose_accessor": {
        const accessor = this.accessor.store.get(data);
        if (!accessor) throw new Error("Accessor not found");
        accessor.dispose();
        break;
      }

      case "create_primitive": {
        if (data.json.material) data.json.material = null;
        this.primitive.create(data.json, data.id);
        break;
      }

      case "change_primitive": {
        if (data.json.material) data.json.material = null;

        const primitive = this.primitive.store.get(data.id);
        if (!primitive) throw new Error(`Primitive not found: ${data.id}`);

        this.primitive.applyJSON(primitive, data.json);
        break;
      }

      case "dispose_primitive": {
        const primitive = this.primitive.store.get(data);
        if (!primitive) throw new Error(`Primitive not found: ${data}`);

        primitive.dispose();
        break;
      }

      case "create_mesh": {
        this.mesh.create(data.json, data.id);
        break;
      }

      case "change_mesh": {
        const mesh = this.mesh.store.get(data.id);
        if (!mesh) throw new Error("Mesh not found");

        this.mesh.applyJSON(mesh, data.json);
        break;
      }

      case "dispose_mesh": {
        const mesh = this.mesh.store.get(data);
        if (!mesh) throw new Error("Mesh not found");
        mesh.dispose();
        break;
      }

      case "create_node": {
        this.builders.node.add(data.json, data.id);
        break;
      }

      case "change_node": {
        this.builders.node.update(data.id, data.json);
        break;
      }

      case "dispose_node": {
        this.builders.node.remove(data);
        break;
      }
    }
  };
}

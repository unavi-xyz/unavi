import { ExtensionProperty, IProperty, PropertyType } from "@gltf-transform/core";
import { Nullable } from "@gltf-transform/core/dist/constants";

import { EXTENSION_NAME } from "../constants";

interface ISpawnPoint extends IProperty {
  title: string;
  team: string;
  group: string;
}

/**
 * Represents a spawn point.
 *
 * @group OMI_spawn_point
 * @see {@link SpawnPointExtension}
 */
export class SpawnPoint extends ExtensionProperty<ISpawnPoint> {
  static override readonly EXTENSION_NAME = EXTENSION_NAME.SpawnPoint;
  declare extensionName: typeof EXTENSION_NAME.SpawnPoint;
  declare propertyType: "SpawnPoint";
  declare parentTypes: [PropertyType.NODE];

  protected init() {
    this.extensionName = EXTENSION_NAME.SpawnPoint;
    this.propertyType = "SpawnPoint";
    this.parentTypes = [PropertyType.NODE];
  }

  protected override getDefaults(): Nullable<ISpawnPoint> {
    return Object.assign(super.getDefaults(), {
      title: "",
      team: "",
      group: "",
    });
  }

  getTitle() {
    return this.get("title");
  }

  setTitle(title: string) {
    this.set("title", title);
  }

  getTeam() {
    return this.get("team");
  }

  setTeam(team: string) {
    this.set("team", team);
  }

  getGroup() {
    return this.get("group");
  }

  setGroup(group: string) {
    this.set("group", group);
  }
}

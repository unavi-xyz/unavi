import { Avatar } from "@unavi/gltf-extensions";
import { Engine, MAX_AVATAR_EQUIP_DISTANCE, RenderEvent } from "engine";
import { useEffect } from "react";

import { HoverState } from "../components/Client";

export function useAvatarEquip(
  engine: Engine | null,
  equippedAvatar: string | null,
  setAvatar: (uri: string) => void,
  setHoverState: (state: HoverState) => void
) {
  useEffect(() => {
    if (!engine) return;

    const listener = (e: RenderEvent) => {
      setHoverState(null);

      if (
        !e.data.isAvatar ||
        !e.data.nodeId ||
        (e.data.distance !== null && e.data.distance > MAX_AVATAR_EQUIP_DISTANCE)
      )
        return;

      const node = engine.scene.node.store.get(e.data.nodeId);
      if (!node) throw new Error(`Node ${e.data.nodeId} not found`);

      const avatar = node.getExtension<Avatar>(Avatar.EXTENSION_NAME);
      if (!avatar) throw new Error(`Node ${e.data.nodeId} has no avatar`);

      const equippable = avatar.getEquippable();
      if (!equippable) return;

      const uri = avatar.getURI();
      const needsBaseURI = uri.startsWith("/");
      const fullURI = needsBaseURI ? `${engine.scene.baseURI}${uri}` : uri;

      // Show tooltip
      if (fullURI === equippedAvatar) setHoverState("avatar_equipped");
      else setHoverState("equip_avatar");
    };

    engine.render.addEventListener("hovered_node", listener);

    return () => {
      engine.render.removeEventListener("hovered_node", listener);
      setHoverState(null);
    };
  }, [engine, equippedAvatar, setHoverState]);

  useEffect(() => {
    if (!engine) return;

    const listener = (e: RenderEvent) => {
      // Ignore click if not pointerlocked
      if (!engine.input.keyboard.isLocked) return;

      if (
        !e.data.isAvatar ||
        !e.data.nodeId ||
        (e.data.distance !== null && e.data.distance > MAX_AVATAR_EQUIP_DISTANCE)
      )
        return;

      const node = engine.scene.node.store.get(e.data.nodeId);
      if (!node) throw new Error(`Node ${e.data.nodeId} not found`);

      const avatar = node.getExtension<Avatar>(Avatar.EXTENSION_NAME);
      if (!avatar) throw new Error(`Node ${e.data.nodeId} has no avatar`);

      const equippable = avatar.getEquippable();
      if (!equippable) return;

      const uri = avatar.getURI();
      const needsBaseURI = uri.startsWith("/");
      const fullURI = needsBaseURI ? `${engine.scene.baseURI}${uri}` : uri;

      if (fullURI === equippedAvatar) return;

      // Equip avatar
      setAvatar(fullURI);
    };

    engine.render.addEventListener("clicked_node", listener);

    return () => {
      engine.render.removeEventListener("clicked_node", listener);
    };
  }, [engine, equippedAvatar, setAvatar]);
}

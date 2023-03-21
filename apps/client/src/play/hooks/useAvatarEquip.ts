import { Avatar } from "@wired-labs/gltf-extensions";
import { Engine, RenderEvent } from "engine";
import { useEffect, useState } from "react";

import { CrosshairAction } from "../CrosshairTooltip";

export function useAvatarEquip(
  engine: Engine | null,
  equippedAvatar: string | null,
  setAvatar: (uri: string) => void
): CrosshairAction {
  const [hoveredNode, setHoveredNode] = useState<string | null>(null);

  useEffect(() => {
    if (!engine) return;

    const listener = (e: RenderEvent) => {
      setHoveredNode(null);

      if (!e.data.isAvatar || !e.data.nodeId) return;

      const node = engine.scene.node.store.get(e.data.nodeId);
      if (!node) throw new Error(`Node ${e.data.nodeId} not found`);

      const avatar = node.getExtension<Avatar>(Avatar.EXTENSION_NAME);
      if (!avatar) throw new Error(`Node ${e.data.nodeId} has no avatar`);

      const equippable = avatar.getEquippable();
      if (!equippable) return;

      const uri = avatar.getURI();
      if (uri === equippedAvatar) return;

      // Show tooltip
      setHoveredNode(e.data.nodeId);
    };

    engine.render.addEventListener("hovered_node", listener);

    return () => {
      engine.render.removeEventListener("hovered_node", listener);
      setHoveredNode(null);
    };
  }, [engine, equippedAvatar]);

  useEffect(() => {
    if (!engine) return;

    const listener = (e: RenderEvent) => {
      // Ignore if not pointerlocked
      if (!engine.input.keyboard.isLocked) return;

      if (!e.data.isAvatar || !e.data.nodeId) return;

      const node = engine.scene.node.store.get(e.data.nodeId);
      if (!node) throw new Error(`Node ${e.data.nodeId} not found`);

      const avatar = node.getExtension<Avatar>(Avatar.EXTENSION_NAME);
      if (!avatar) throw new Error(`Node ${e.data.nodeId} has no avatar`);

      const equippable = avatar.getEquippable();
      if (!equippable) return;

      const uri = avatar.getURI();
      if (uri === equippedAvatar) return;

      // Equip avatar
      setAvatar(uri);
    };

    engine.render.addEventListener("clicked_node", listener);

    return () => {
      engine.render.removeEventListener("clicked_node", listener);
    };
  }, [engine, equippedAvatar, setAvatar]);

  return hoveredNode ? "equip_avatar" : null;
}

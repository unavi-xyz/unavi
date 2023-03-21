import { Avatar } from "@wired-labs/gltf-extensions";
import { RenderEvent } from "engine";
import { useEffect, useState } from "react";

import { CrosshairAction } from "../CrosshairTooltip";
import { usePlayStore } from "../store";

export function useAvatarEquip(setAvatar: (uri: string) => void): CrosshairAction {
  const engine = usePlayStore((state) => state.engine);
  const equippedAvatarUri = usePlayStore((state) => state.avatar);
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
      if (uri === equippedAvatarUri) return;

      // Show tooltip
      setHoveredNode(e.data.nodeId);
    };

    engine.render.addEventListener("hovered_node", listener);

    return () => {
      engine.render.removeEventListener("hovered_node", listener);
      setHoveredNode(null);
    };
  }, [engine, equippedAvatarUri]);

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

      // Equip avatar
      const uri = avatar.getURI();
      setAvatar(uri);
    };

    engine.render.addEventListener("clicked_node", listener);

    return () => {
      engine.render.removeEventListener("clicked_node", listener);
    };
  }, [engine, setAvatar]);

  return hoveredNode ? "equip_avatar" : null;
}

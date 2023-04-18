import { ExtensibleProperty, ExtensionProperty } from "@gltf-transform/core";
import { AudioEmitter, Avatar, Collider, SpawnPoint } from "@unavi/gltf-extensions";
import { useEffect, useState } from "react";

export function useExtension<T extends ExtensionProperty>(
  property: ExtensibleProperty | null,
  extensionName: string
) {
  const [extension, setExtension] = useState<T | null>(null);

  useEffect(() => {
    if (!property) {
      setExtension(null);
      return;
    }

    const extension = property.getExtension<T>(extensionName);
    setExtension(extension);
  }, [property, extensionName]);

  return extension;
}

export function useAudioEmitter(property: ExtensibleProperty | null) {
  return useExtension<AudioEmitter>(property, AudioEmitter.EXTENSION_NAME);
}

export function useAvatar(property: ExtensibleProperty | null) {
  return useExtension<Avatar>(property, Avatar.EXTENSION_NAME);
}

export function useCollider(property: ExtensibleProperty | null) {
  return useExtension<Collider>(property, Collider.EXTENSION_NAME);
}

export function useSpawnPoint(property: ExtensibleProperty | null) {
  return useExtension<SpawnPoint>(property, SpawnPoint.EXTENSION_NAME);
}

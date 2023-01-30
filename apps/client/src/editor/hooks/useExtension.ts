import { ExtensibleProperty, ExtensionProperty } from "@gltf-transform/core";
import { Collider, ColliderExtension } from "engine";
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

export function useCollider(property: ExtensibleProperty | null) {
  return useExtension<Collider>(property, ColliderExtension.EXTENSION_NAME);
}

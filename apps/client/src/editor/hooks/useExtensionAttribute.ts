import { ExtensionProperty } from "@gltf-transform/core";
import { useEffect, useState } from "react";

export function useExtensionAttribute<T extends ExtensionProperty, A extends keyof T>(
  extension: T | null,
  attribute: A
): T[A] | null {
  const [value, setValue] = useState<T[A] | null>(null);

  useEffect(() => {
    if (!extension) {
      setValue(null);
      return;
    }

    setValue(extension[attribute]);

    const onChange = (e: any) => {
      if (e.attribute !== attribute) return;

      const newValue = extension[attribute];
      setValue(newValue);
    };

    extension.addEventListener("change", onChange);

    return () => {
      extension.removeEventListener("change", onChange);
    };
  }, [extension, attribute]);

  return value;
}

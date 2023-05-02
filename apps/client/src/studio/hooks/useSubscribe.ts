import { ExtensibleProperty, ExtensionProperty } from "@gltf-transform/core";
import { subscribe, SubscribeReturnValues, SubscribeValues } from "engine";
import { useEffect, useState } from "react";

type NotNull<T> = T extends null ? never : T;

export function useSubscribe<
  Property extends ExtensibleProperty | ExtensionProperty | null,
  Key extends SubscribeValues<NotNull<Property>>
>(property: Property, attribute: Key) {
  const [value, setValue] =
    useState<Property extends null ? undefined : SubscribeReturnValues<NotNull<Property>, Key>>();

  useEffect(() => {
    if (property === null) return;

    // @ts-ignore
    const unsubscribe = subscribe(property, attribute, (newValue) => {
      // @ts-ignore
      setValue(newValue);
    });

    return () => {
      unsubscribe();
      setValue(undefined);
    };
  }, [property, attribute]);

  return value;
}

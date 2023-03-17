import { ExtensibleProperty, ExtensionProperty } from "@gltf-transform/core";
import { GraphNodeEvent } from "property-graph";

type Property = ExtensibleProperty | ExtensionProperty;

type Split<S extends string, D extends string> = string extends S
  ? string[]
  : S extends ""
  ? []
  : S extends `${infer T}${D}${infer U}`
  ? [T, ...Split<U, D>]
  : [S];

type FirstNotUndefined<T extends any[]> = T extends [infer F, ...infer R]
  ? F extends undefined
    ? FirstNotUndefined<R>
    : F
  : never;

type ChangeValue<P extends Property, Start extends string> = Split<
  Extract<keyof P, string>,
  Start
>[1];

type GetValue<P extends Property> = ChangeValue<P, "get">;
type ListValue<P extends Property> = Exclude<ChangeValue<P, "list">, "ener">;

export type SubscribeValues<P extends Property> = FirstNotUndefined<[GetValue<P>, ListValue<P>]>;

type ReturnValue<
  Start extends string,
  P extends Property,
  Key extends ChangeValue<P, Start>
  // @ts-ignore
> = Key extends string ? ReturnType<P[`${Start}${Capitalize<Key>}`]> : undefined;

type GetReturnValue<P extends Property, Key extends GetValue<P>> = ReturnValue<"get", P, Key>;
type ListReturnValue<P extends Property, Key extends ListValue<P>> = ReturnValue<"list", P, Key>;

export type SubscribeReturnValues<
  P extends Property,
  Key extends SubscribeValues<P>
> = Key extends GetValue<P>
  ? GetReturnValue<P, Key>
  : Key extends ListValue<P>
  ? ListReturnValue<P, Key>
  : never;

type CallbackReturn = void | (() => void) | Promise<void> | Promise<() => void>;

/**
 * Subscribe to changes on a property.
 *
 * @param property - The property.
 * @param attribute - The attribute to subscribe to.
 * @param callback - The callback to invoke when the attribute changes. Can return a function that is called after the next change (useful for cleanup).
 * @returns A function to unsubscribe.
 */
export function subscribe<P extends Property, Key extends SubscribeValues<P>>(
  property: P,
  attribute: Key,
  callback: (value: SubscribeReturnValues<P, Key>) => CallbackReturn
): () => void {
  let cleanup: (() => void) | void;

  const onChange = async (e: GraphNodeEvent) => {
    if (e.attribute === attribute.toLowerCase()) {
      // Call cleanup function
      if (cleanup) {
        cleanup();
        cleanup = undefined;
      }

      // @ts-ignore
      let get = property[`get${attribute}`]?.bind(property);

      if (get === undefined) {
        // @ts-ignore
        get = property[`list${attribute}`]?.bind(property);
      }

      const value = get();

      cleanup = await callback(value);
    }
  };

  // Get initial value
  onChange({ type: "change", target: property, attribute: attribute.toLowerCase() });

  // Listen for changes
  property.addEventListener("change", onChange);

  // Cleanup on dispose
  const onDispose = () => {
    if (cleanup) cleanup();
  };

  property.addEventListener("dispose", onDispose);

  return () => {
    property.removeEventListener("change", onChange);
    property.removeEventListener("dispose", onDispose);
  };
}

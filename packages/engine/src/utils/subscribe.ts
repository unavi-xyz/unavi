import { ExtensibleProperty } from "@gltf-transform/core";
import { GraphNodeEvent } from "property-graph";

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

type ChangeValue<Property extends ExtensibleProperty, Start extends string> = Split<
  Extract<keyof Property, string>,
  Start
>[1];

type GetValue<Property extends ExtensibleProperty> = ChangeValue<Property, "get">;
type ListValue<Property extends ExtensibleProperty> = Exclude<
  ChangeValue<Property, "list">,
  "ener"
>;

type Values<Property extends ExtensibleProperty> = FirstNotUndefined<
  [GetValue<Property>, ListValue<Property>]
>;

type ReturnValue<
  Start extends string,
  Property extends ExtensibleProperty,
  Key extends ChangeValue<Property, Start>
  // @ts-ignore
> = Key extends string ? ReturnType<Property[`${Start}${Capitalize<Key>}`]> : undefined;

type GetReturnValue<
  Property extends ExtensibleProperty,
  Key extends GetValue<Property>
> = ReturnValue<"get", Property, Key>;

type ListReturnValue<
  Property extends ExtensibleProperty,
  Key extends ListValue<Property>
> = ReturnValue<"list", Property, Key>;

type ReturnValues<
  Property extends ExtensibleProperty,
  Key extends Values<Property>
> = Key extends GetValue<Property>
  ? GetReturnValue<Property, Key>
  : Key extends ListValue<Property>
  ? ListReturnValue<Property, Key>
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
export function subscribe<Property extends ExtensibleProperty, Key extends Values<Property>>(
  property: Property,
  attribute: Key,
  callback: (value: ReturnValues<Property, Key>) => CallbackReturn
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

  property.addEventListener("change", onChange);

  return () => {
    property.removeEventListener("change", onChange);
  };
}

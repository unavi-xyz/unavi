import { BehaviorVariable } from "@wired-labs/gltf-extensions";
import { useEffect, useState } from "react";

export function useVariableAttribute<T extends BehaviorVariable, A extends keyof T>(
  variable: T | null,
  attribute: A
): T[A] | null {
  const [value, setValue] = useState<T[A] | null>(null);

  useEffect(() => {
    if (!variable) {
      setValue(null);
      return;
    }

    setValue(variable[attribute]);

    const onChange = (e: any) => {
      if (e.attribute !== attribute) return;

      const newValue = variable[attribute];
      setValue(newValue);
    };

    variable.addEventListener("change", onChange);

    return () => {
      variable.removeEventListener("change", onChange);
    };
  }, [variable, attribute]);

  return value;
}

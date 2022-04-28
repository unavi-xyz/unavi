import { useEffect, useState } from "react";

function getPastelColorFromSeed(seed: string): string {
  const hash = seed.split("").reduce((a, b) => {
    a = (a << 5) - a + b.charCodeAt(0);
    return a & a;
  }, 0);
  return `hsl(${hash % 360}, 80%, 70%)`;
}

export function useColorFromSeed(seed: string | undefined) {
  const [color, setColor] = useState<string>();

  useEffect(() => {
    if (!seed) {
      setColor(undefined);
      return;
    }

    setColor(getPastelColorFromSeed(seed));
  }, [seed]);

  return color;
}

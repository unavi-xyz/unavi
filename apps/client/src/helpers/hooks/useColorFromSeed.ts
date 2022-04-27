import { useEffect, useState } from "react";

function getBrightColorFromSeed(seed: string): string {
  const hash = seed.split("").reduce((a, b) => {
    a = (a << 5) - a + b.charCodeAt(0);
    return a & a;
  }, 0);
  return `hsl(${hash % 360}, 100%, 50%)`;
}

export function useColorFromSeed(seed: string) {
  const [color, setColor] = useState<string>();

  useEffect(() => {
    if (!seed) {
      setColor(undefined);
      return;
    }

    setColor(getBrightColorFromSeed(seed));
  }, [seed]);

  return color;
}

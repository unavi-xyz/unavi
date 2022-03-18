import { useEffect, useState } from "react";

export function useIsMobile() {
  const hasWindow = typeof window !== "undefined";

  const [isMobile, setIsMobile] = useState(false);

  useEffect(() => {
    function getWindowDimensions() {
      const width = hasWindow ? window.innerWidth : null;
      const height = hasWindow ? window.innerHeight : null;
      return {
        width,
        height,
      };
    }

    function handleResize() {
      const dimensions = getWindowDimensions();
      const newIsMobile = dimensions.width < 640;
      if (newIsMobile !== isMobile) setIsMobile(newIsMobile);
    }

    if (hasWindow) {
      handleResize();
      window.addEventListener("resize", handleResize);
      return () => {
        window.removeEventListener("resize", handleResize);
      };
    }
  }, [hasWindow, isMobile]);

  return isMobile;
}

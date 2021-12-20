import { useState, useEffect } from "react";

function getWindowDimensions() {
  const { innerWidth: width, innerHeight: height } = window;
  return {
    width,
    height,
    isMobile: width < 600,
    isXs: width < 900,
  };
}

export function useWindowDimensions() {
  const [windowDimensions, setWindowDimensions] = useState({
    width: 1920,
    height: 1080,
    isMobile: true,
    isXs: true,
  });

  function handleResize() {
    setWindowDimensions(getWindowDimensions());
  }

  useEffect(() => {
    handleResize();
    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, []);

  return windowDimensions;
}

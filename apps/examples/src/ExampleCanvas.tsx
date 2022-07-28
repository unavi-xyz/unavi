import { useEffect, useRef, useState } from "react";
import { AnimationAction, AnimationMixer, Scene } from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader";

import { Engine, IGLTF } from "@wired-xr/new-engine";

import Panel from "./Panel/Panel";

export interface RenderInfo {
  load: {
    time: number;
    threeTime?: number;
    exportedTime?: number;
  };
  memory: {
    geometries: number;
    textures: number;
  };
  render: {
    calls: number;
    lines: number;
    points: number;
    triangles: number;
  };
}

export interface Settings {
  testThree: boolean;
  testExport: boolean;
}

interface Props {
  uri: string;
}

export default function ExampleCanvas({ uri }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const [loaded, setLoaded] = useState(false);
  const [animations, setAnimations] = useState<AnimationAction[]>();
  const [info, setInfo] = useState<RenderInfo>();
  const [settings, setSettings] = useState<Settings>({
    testThree: false,
    testExport: true,
  });

  useEffect(() => {
    if (!canvasRef.current) return;

    // Set canvas size
    updateCanvasSize();
    window.addEventListener("resize", updateCanvasSize);

    // Create engine
    const engine = new Engine({ canvas: canvasRef.current });

    // Move stats to the right
    const stats = document.getElementById("stats");
    if (stats) {
      stats.style.left = "calc(100% - 79px)";
    }

    let interval: NodeJS.Timer;
    let startTime = performance.now();
    let endTime: number;
    let exportedStartTime: number;
    let exportedTime: number;

    async function handleLoaded({ animations }: IGLTF) {
      setAnimations(animations);
      setLoaded(true);

      // Start first animation
      if (animations.length > 0) {
        animations[0].play();
      }

      async function testThree() {
        if (!settings.testThree) return;

        // Test against threejs loader load time
        const threeScene = new Scene();
        const startThreeTime = performance.now();
        const threeLoader = new GLTFLoader();
        const threeGltf = await threeLoader.loadAsync(uri);
        const threeMixer = new AnimationMixer(threeGltf.scene);
        threeGltf.animations.forEach((animation) => threeMixer.clipAction(animation));
        threeScene.add(threeGltf.scene);
        const time = Math.round(performance.now() - startThreeTime) / 1000;
        return time;
      }

      const threeTime = await testThree();

      function updateInfo() {
        const { render, memory } = engine.info();
        const { triangles, points, lines, calls } = render;

        setInfo({
          load: {
            time: endTime,
            threeTime,
            exportedTime,
          },
          memory,
          render: {
            calls,
            lines,
            points,
            triangles,
          },
        });
      }

      updateInfo();
      interval = setInterval(updateInfo, 1000);
    }

    // Load gltf
    engine
      .loadGltf(uri)
      .then(async (res) => {
        endTime = Math.round(performance.now() - startTime) / 1000;

        if (settings.testExport) {
          // Export to glb
          const glb = await engine.export();
          const blob = new Blob([glb], { type: "application/octet-stream" });
          const url = URL.createObjectURL(blob);

          // Load the exported glb
          exportedStartTime = performance.now();
          const gltf = await engine.loadGltf(url);
          exportedTime = Math.round(performance.now() - exportedStartTime) / 1000;

          handleLoaded(gltf);
          return;
        }

        handleLoaded(res);
      })
      .catch((error) => {
        console.error(error);
        setLoaded(false);
      });

    return () => {
      engine.destroy();
      window.removeEventListener("resize", updateCanvasSize);
      setAnimations(undefined);
      setLoaded(false);
      setInfo(undefined);
      clearInterval(interval);
    };
  }, [canvasRef, uri, settings]);

  function updateCanvasSize() {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const container = containerRef.current;
    if (!container) return;

    // Resize canvas
    canvas.width = container.clientWidth;
    canvas.height = container.clientHeight;
  }

  return (
    <>
      <div ref={containerRef} className="relative w-full h-full overflow-hidden">
        <canvas ref={canvasRef} className="w-full h-full" />
      </div>

      <Panel
        loaded={loaded}
        animations={animations}
        info={info}
        settings={settings}
        setSettings={setSettings}
      />
    </>
  );
}

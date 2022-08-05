import { useEffect, useRef, useState } from "react";
import { AnimationAction, AnimationMixer, Box3, Scene, Vector3 } from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls";
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
    testExport: false,
  });

  useEffect(() => {
    if (!canvasRef.current) return;

    // Set canvas size
    updateCanvasSize();
    window.addEventListener("resize", updateCanvasSize);

    // Create engine
    const engine = new Engine({ canvas: canvasRef.current, stats: true, alpha: true });

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

      const camera = engine.camera;

      // Calculate bounding box
      const boundingBox = new Box3().setFromObject(engine.scene);
      const size = boundingBox.getSize(new Vector3());
      const center = boundingBox.getCenter(new Vector3());

      // Set camera position
      if (size.x === 0) size.setX(1);
      if (size.y === 0) size.setY(1);
      if (size.z === 0) size.setZ(1);
      camera.position.set(size.x, size.y, size.z * 2);

      // Set camera rotation
      camera.lookAt(center);
      const orbitControls = new OrbitControls(camera, engine.renderer.domElement);
      orbitControls.target.copy(center);

      // Set camera near and far
      const min = boundingBox.min.z === 0 ? 1 : boundingBox.min.z;
      const max = boundingBox.max.z === 0 ? 1 : boundingBox.max.z;
      camera.near = Math.abs(min) / 1000;
      camera.far = Math.abs(max) * 100;
      camera.far = Math.max(camera.far, 50);
      camera.updateProjectionMatrix();

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
          const exportStart = performance.now();
          const glb = await engine.export();
          const blob = new Blob([glb], { type: "application/octet-stream" });
          const url = URL.createObjectURL(blob);
          const exportDone = Math.round(performance.now() - exportStart) / 1000;
          console.log("Exported in", exportDone, "s");

          // Load the exported glb
          exportedStartTime = performance.now();
          const gltf = await engine.loadGltf(url);
          exportedTime = Math.round(performance.now() - exportedStartTime) / 1000;
          console.log("Loaded exported in", exportedTime, "s");

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

import { Physics } from "@react-three/cannon";
import { Canvas } from "@react-three/fiber";
import produce from "immer";
import Head from "next/head";
import Link from "next/link";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";
import { MdClose } from "react-icons/md";

import { InstancedScene } from "@wired-xr/scene";

import Player from "../../src/components/app/Player";
import { getFileByPath } from "../../src/helpers/studio/filesystem";
import { useProject } from "../../src/helpers/studio/hooks/useProject";
import { useStudioStore } from "../../src/helpers/studio/store";
import { Project } from "../../src/helpers/studio/types";

export default function Preview() {
  const router = useRouter();
  const project = useProject();
  const root = useStudioStore((state) => state.rootHandle);

  const [loadedProject, setLoadedProject] = useState<Project>();

  useEffect(() => {
    if (!root) {
      router.push("/studio");
    }
  }, [root, router]);

  useEffect(() => {
    async function loadProject() {
      if (!project) {
        setLoadedProject(undefined);
        return;
      }

      try {
        const newProject = await produce(project, async (draft) => {
          //load assets
          await Promise.all(
            Object.entries(draft.scene.assets).map(async ([id, asset]) => {
              //get file
              const root = useStudioStore.getState().rootHandle;
              if (!root) throw new Error("No root directory");
              const fileHandle = await getFileByPath(asset.uri, root);
              if (!fileHandle) throw new Error("File not found");
              const file = await fileHandle.getFile();
              if (!file) throw new Error("Failed to read file");

              if (asset.type === "image" || asset.type === "model") {
                const url = URL.createObjectURL(file);
                draft.scene.assets[id].data = url;
              } else if (asset.type === "material") {
                const text = await file.text();
                const json = JSON.parse(text);
                draft.scene.assets[id].data = json;
              }
            })
          );
        });

        setLoadedProject(newProject);
      } catch (err) {
        console.error(err);
        setLoadedProject(undefined);
      }
    }

    loadProject();
  }, [project]);

  if (!loadedProject) return null;

  return (
    <div className="h-full">
      <Head>
        <title>{loadedProject.name} / The Wired </title>
      </Head>

      <div className="crosshair" />

      <Canvas shadows className="w-full h-full">
        <Physics>
          <InstancedScene scene={loadedProject.scene} />

          <Player />
        </Physics>
      </Canvas>

      <div
        onClick={(e) => e.stopPropagation()}
        className="fixed top-0 right-0 p-6 text-2xl"
      >
        <Link href={"/studio"} passHref>
          <div
            className="cursor-pointer p-2 rounded-full bg-surface text-onSurface
                       backdrop-blur bg-opacity-60 hover:bg-opacity-100 transition active:bg-opacity-90"
          >
            <MdClose />
          </div>
        </Link>
      </div>
    </div>
  );
}

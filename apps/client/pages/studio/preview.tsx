import { Physics, Triplet } from "@react-three/cannon";
import { Canvas } from "@react-three/fiber";
import produce from "immer";
import Link from "next/link";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";
import { MdClose } from "react-icons/md";

import { Player, Scene, traverseTree } from "@wired-xr/engine";

import ToggleDebug from "../../src/studio/components/StudioCanvas/ToggleDebug";
import { getFileByPath } from "../../src/studio/filesystem";
import { useProject } from "../../src/studio/hooks/useProject";
import { useStudioStore } from "../../src/studio/store";
import { Project } from "../../src/studio/types";
import MetaTags from "../../src/ui/MetaTags";

export default function Preview() {
  const router = useRouter();
  const project = useProject();
  const root = useStudioStore((state) => state.rootHandle);
  const debug = useStudioStore((state) => state.debug);

  const [loadedProject, setLoadedProject] = useState<Project>();
  const [spawn, setSpawn] = useState<Triplet>();

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

        //set spawn
        const spawns: Triplet[] = [];
        traverseTree(newProject.scene.tree, (entity) => {
          if (entity.type === "Spawn") spawns.push(entity.transform.position);
        });
        if (spawns.length > 0) setSpawn(spawns[0]);

        //load scene
        setLoadedProject(newProject);
      } catch (err) {
        console.error(err);
        setLoadedProject(undefined);
        setSpawn(undefined);
      }
    }

    loadProject();
  }, [project]);

  return (
    <>
      <MetaTags title="Studio" />

      <div className="h-full">
        <div className="crosshair" />

        {loadedProject && (
          <Canvas shadows className="w-full h-full">
            <Physics>
              <ToggleDebug enabled={debug}>
                <Scene scene={loadedProject.scene} />
                <Player spawn={spawn} />
              </ToggleDebug>
            </Physics>
          </Canvas>
        )}

        <div
          onClick={(e) => e.stopPropagation()}
          className="fixed top-0 right-0 p-6 text-2xl"
        >
          <Link href={"/studio"} passHref>
            <a
              className="block cursor-pointer p-2 rounded-full bg-surface text-onSurface
                         backdrop-blur bg-opacity-60 hover:bg-opacity-100 transition active:bg-opacity-90"
            >
              <MdClose />
            </a>
          </Link>
        </div>
      </div>
    </>
  );
}

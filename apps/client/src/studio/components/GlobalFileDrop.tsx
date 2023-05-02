"use client";

import { useStudio } from "./Studio";

interface Props {
  children: React.ReactNode;
}

export default function GlobalFileDrop({ children }: Props) {
  const { engine } = useStudio();

  return (
    <div
      className="h-full w-full"
      onDragOver={(e) => e.preventDefault()}
      onDrop={async (e) => {
        if (!engine) return;

        e.preventDefault();

        const files = Array.from(e.dataTransfer.files);

        const glbs = files.filter((file) => file.name.endsWith(".glb"));
        const other = files.filter((file) => !file.name.endsWith(".glb"));

        glbs.forEach((file) => engine.scene.addFile(file));
        if (other.length) engine.scene.addFiles(other);
      }}
    >
      {children}
    </div>
  );
}

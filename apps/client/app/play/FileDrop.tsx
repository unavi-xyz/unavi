import { useEffect, useState } from "react";

import { useSetAvatar } from "@/src/play/hooks/useSetAvatar";

export default function FileDrop() {
  const [isDragging, setIsDragging] = useState(false);

  const setAvatar = useSetAvatar();

  useEffect(() => {
    const onDragOver = (e: DragEvent) => {
      e.preventDefault();
      setIsDragging(true);
    };

    const onDragLeave = (e: DragEvent) => {
      e.preventDefault();
      setIsDragging(false);
    };

    const onDrop = (e: DragEvent) => {
      e.preventDefault();
      setIsDragging(false);

      const file = e.dataTransfer?.files[0];
      if (!file) return;

      const isVRM = file.name.endsWith(".vrm");
      if (!isVRM) return;

      const url = URL.createObjectURL(file);
      setAvatar(url);
    };

    document.body.addEventListener("dragover", onDragOver);
    document.body.addEventListener("dragleave", onDragLeave);
    document.body.addEventListener("drop", onDrop);

    return () => {
      document.body.removeEventListener("dragover", onDragOver);
      document.body.removeEventListener("dragleave", onDragLeave);
      document.body.removeEventListener("drop", onDrop);
    };
  }, [setAvatar]);

  if (!isDragging) return null;

  return (
    <div className="pointer-events-none fixed inset-0 z-50 h-screen w-screen bg-black/10 backdrop-blur">
      <div className="flex h-full w-full items-center justify-center">
        <div className="rounded-xl bg-white p-8 text-xl shadow">
          Drop <span className="font-bold">.VRM</span> file to equip avatar
        </div>
      </div>
    </div>
  );
}

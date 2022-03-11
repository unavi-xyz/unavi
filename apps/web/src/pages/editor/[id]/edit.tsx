import { useEffect } from "react";
import { IoMdClose } from "react-icons/io";
import { useRouter } from "next/router";
import Head from "next/head";
import { useAtom } from "jotai";

import Navbar from "../../../components/editor/Navbar/Navbar";
import EditorSidebar from "../../../components/editor/EditorSidebar/EditorSidebar";
import EditorCanvas from "../../../components/editor/EditorCanvas/EditorCanvas";
import { previewModeAtom, worldIdAtom } from "../../../helpers/editor/state";
import PreviewCanvas from "../../../components/editor/EditorCanvas/PreviewCanvas";

export default function Id() {
  const router = useRouter();
  const id = router.query.id as string;

  const [, setWorldId] = useAtom(worldIdAtom);
  const [previewMode, setPreviewMode] = useAtom(previewModeAtom);

  useEffect(() => {
    setWorldId(id);
  }, [id, setWorldId]);

  function handleClose() {
    setPreviewMode(false);
  }

  return (
    <>
      <Head>
        <title>The Wired - Editor</title>
      </Head>

      {previewMode ? (
        <div className="relative h-full">
          <PreviewCanvas />

          <div className="crosshair" />

          <div
            onClick={handleClose}
            className="absolute top-6 right-6 hover:cursor-pointer bg-black rounded-full p-1.5
                       bg-opacity-30 hover:bg-opacity-40 backdrop-blur-sm"
          >
            <IoMdClose className="text-3xl text-white" />
          </div>
        </div>
      ) : (
        <div className="h-full">
          <Navbar id={id} />

          <div className="h-full bg-neutral-200 flex">
            <EditorCanvas />

            <div className="h-full w-full max-w-md">
              <EditorSidebar />
            </div>
          </div>
        </div>
      )}
    </>
  );
}

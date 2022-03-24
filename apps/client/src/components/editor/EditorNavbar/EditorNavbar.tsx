import { IoMdEye } from "react-icons/io";
import { MdArrowBackIosNew } from "react-icons/md";
import { HiOutlineCubeTransparent, HiOutlineCube } from "react-icons/hi";
import { useRouter } from "next/router";

import { mergeLocalScene } from "../../scene/localScenes/db";
import { useLocalScene } from "../../scene/localScenes/useLocalScene";
import { editorManager, useStore } from "../helpers/store";

import MiddleButtons from "./MiddleButtons";
import { Tooltip } from "../../base";

interface Props {
  id: string;
}

export default function EditorNavbar({ id }: Props) {
  const router = useRouter();
  const localScene = useLocalScene(id);

  const debugMode = useStore((state) => state.debugMode);

  async function handleBack() {
    const canvas = document.querySelector("canvas");
    const image = canvas.toDataURL("image/jpeg", 0.5);
    await mergeLocalScene(id, { image });
    router.push(`/editor/${id}`);
  }

  function toggleDebug() {
    editorManager.setDebugMode(!debugMode);
  }

  function handlePreview() {
    editorManager.setSelected(undefined);
    editorManager.setPreviewMode(true);
  }

  return (
    <div
      className="w-screen h-12 bg-white flex items-center justify-between px-2
                    border-b-[1px] border-neutral-200"
    >
      <div className="flex items-center space-x-2 w-1/3">
        <div
          onClick={handleBack}
          className="hover:cursor-pointer text-xl p-2 rounded-full"
        >
          <MdArrowBackIosNew />
        </div>

        <div className="text-lg">{localScene?.name ?? id}</div>
      </div>

      <div className="w-1/3 flex justify-center text-2xl">
        <MiddleButtons />
      </div>

      <div className="w-1/3 flex justify-end space-x-2 pr-2">
        <Tooltip
          text={debugMode ? "Disable Debug" : "Enable Debug"}
          placement="bottom"
        >
          <div
            onClick={toggleDebug}
            className="w-10 h-10 rounded-md flex items-center justify-center
                       hover:cursor-pointer"
          >
            {debugMode ? (
              <HiOutlineCube className="text-xl" />
            ) : (
              <HiOutlineCubeTransparent className="text-xl" />
            )}
          </div>
        </Tooltip>

        <Tooltip text="Preview" placement="bottom">
          <div
            onClick={handlePreview}
            className="w-10 h-10 rounded-md flex items-center justify-center
          hover:cursor-pointer"
          >
            <IoMdEye className="text-2xl" />
          </div>
        </Tooltip>
      </div>
    </div>
  );
}

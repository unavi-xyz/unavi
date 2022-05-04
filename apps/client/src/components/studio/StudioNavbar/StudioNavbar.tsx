import { useEffect } from "react";
import { MdArrowBackIosNew, MdPreview } from "react-icons/md";
import { useRouter } from "next/router";
import { VscMove, VscSync } from "react-icons/vsc";
import { CgArrowsExpandUpRight } from "react-icons/cg";
import Link from "next/link";

import { useLocalSpace } from "../../../helpers/indexedDB/localSpaces/hooks/useLocalScene";
import { updateLocalSpace } from "../../../helpers/indexedDB/localSpaces/helpers";

import Tooltip from "../../base/Tooltip";
import ToolButton from "./ToolButton";
import IconButton from "../../base/IconButton";

export default function StudioNavbar() {
  const router = useRouter();
  const id = router.query.id as string;

  const localSpace = useLocalSpace(id);

  useEffect(() => {
    if (id) router.prefetch(`/create/${id}`);
  }, [id, router]);

  async function handleBack() {
    //take a screenshot of the scene before navigating
    const canvas = document.querySelector("canvas");
    if (canvas) {
      const image = canvas.toDataURL("image/jpeg");
      await updateLocalSpace(id, { image });
    }

    router.push(`/create/${id}`);
  }

  return (
    <div className="flex justify-between items-center h-full px-4">
      <div className="w-full flex items-center space-x-4 text-lg">
        <div onClick={handleBack} className="cursor-pointer">
          <MdArrowBackIosNew />
        </div>

        <div>{localSpace?.name}</div>
      </div>

      <div className="w-full h-full flex justify-center items-center space-x-2 p-2">
        <ToolButton tool="translate">
          <VscMove />
        </ToolButton>
        <ToolButton tool="rotate">
          <VscSync />
        </ToolButton>
        <ToolButton tool="scale">
          <CgArrowsExpandUpRight />
        </ToolButton>
      </div>

      <div className="w-full h-full flex justify-end items-center p-2">
        <Tooltip text="Preview" placement="bottom">
          <Link href={`/studio/${id}/preview`} passHref>
            <div className="h-full">
              <IconButton>
                <MdPreview />
              </IconButton>
            </div>
          </Link>
        </Tooltip>
      </div>
    </div>
  );
}

import Link from "next/link";
import { useRouter } from "next/router";
import { useEffect } from "react";
import { BiMove } from "react-icons/bi";
import { CgArrowsExpandUpRight } from "react-icons/cg";
import { MdArrowBackIosNew, MdPreview, MdSync } from "react-icons/md";

import { updateLocalSpace } from "../../../helpers/indexeddb/LocalSpace/helpers";
import { useLocalSpace } from "../../../helpers/indexeddb/LocalSpace/hooks/useLocalScene";
import { useLensStore } from "../../../helpers/lens/store";
import IconButton from "../../base/IconButton";
import Tooltip from "../../base/Tooltip";
import LoginButton from "../../layouts/NavbarLayout/LoginButton";
import PublishButton from "./PublishButton";
import ToolButton from "./ToolButton";

export default function StudioNavbar() {
  const router = useRouter();
  const id = router.query.id as string;

  const handle = useLensStore((state) => state.handle);
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
    <div className="flex justify-between items-center h-full px-4 py-2">
      <div className="w-full flex items-center space-x-4 text-lg">
        <div onClick={handleBack} className="cursor-pointer">
          <MdArrowBackIosNew />
        </div>

        <div>{localSpace?.name}</div>
      </div>

      <div className="w-full h-full flex justify-center items-center space-x-2">
        <ToolButton tool="translate">
          <BiMove />
        </ToolButton>
        <ToolButton tool="rotate">
          <MdSync />
        </ToolButton>
        <ToolButton tool="scale">
          <CgArrowsExpandUpRight />
        </ToolButton>
      </div>

      <div className="w-full h-full flex justify-end items-center space-x-4">
        <div className="h-full">
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

        <div className="text-sm">
          {handle ? <PublishButton /> : <LoginButton />}
        </div>
      </div>
    </div>
  );
}

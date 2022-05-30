import Link from "next/link";
import { BiMove } from "react-icons/bi";
import { CgArrowsExpandUpRight } from "react-icons/cg";
import { HiCubeTransparent } from "react-icons/hi";
import { MdArrowBackIosNew, MdPreview, MdSync } from "react-icons/md";

import { useLensStore } from "../../../helpers/lens/store";
import { useProject } from "../../../helpers/studio/hooks/useProject";
import { useStudioStore } from "../../../helpers/studio/store";
import IconButton from "../../base/IconButton";
import Tooltip from "../../base/Tooltip";
import LoginButton from "../../layouts/NavbarLayout/LoginButton";
import PublishButton from "./PublishButton";
import ToolButton from "./ToolButton";

export default function StudioNavbar() {
  const handle = useLensStore((state) => state.handle);
  const debug = useStudioStore((state) => state.debug);
  const project = useProject();

  function handleToggleDebug() {
    useStudioStore.setState({ debug: !debug, selectedId: undefined });
  }

  return (
    <div className="flex justify-between items-center h-full px-4 py-2">
      <div className="w-full flex items-center space-x-4 text-lg">
        <Link href="/create">
          <div className="cursor-pointer transition text-outline hover:text-inherit p-1">
            <MdArrowBackIosNew />
          </div>
        </Link>

        <div>{project?.name}</div>
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
        <div className="h-full aspect-square">
          <Tooltip
            text={`${debug ? "Disable" : "Enable"} Debug`}
            placement="bottom"
          >
            <IconButton selected={debug} onClick={handleToggleDebug}>
              <HiCubeTransparent />
            </IconButton>
          </Tooltip>
        </div>

        <div className="h-full aspect-square">
          <Tooltip text="Preview" placement="bottom">
            <Link href="/studio/preview" passHref>
              <div className="h-full">
                <IconButton>
                  <MdPreview />
                </IconButton>
              </div>
            </Link>
          </Tooltip>
        </div>

        {handle ? <PublishButton /> : <LoginButton />}
      </div>
    </div>
  );
}

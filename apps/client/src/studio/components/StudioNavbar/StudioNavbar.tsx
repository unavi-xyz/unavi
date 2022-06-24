import Link from "next/link";
import { useRouter } from "next/router";
import { BiMove } from "react-icons/bi";
import { CgArrowsExpandUpRight } from "react-icons/cg";
import { HiCubeTransparent } from "react-icons/hi";
import {
  MdArrowBackIosNew,
  MdOutlineGridOn,
  MdPreview,
  MdSync,
} from "react-icons/md";

import LoginButton from "../../../home/layouts/NavbarLayout/LoginButton";
import { useLensStore } from "../../../lib/lens/store";
import { useStudioStore } from "../../../studio/store";
import IconButton from "../../../ui/base/IconButton";
import Tooltip from "../../../ui/base/Tooltip";
import { useProject } from "../../hooks/useProject";
import PublishButton from "./PublishButton";
import ToolButton from "./ToolButton";

export default function StudioNavbar() {
  const handle = useLensStore((state) => state.handle);
  const debug = useStudioStore((state) => state.debug);
  const grid = useStudioStore((state) => state.grid);

  const project = useProject();
  const router = useRouter();

  function handleToggleDebug() {
    useStudioStore.setState({ debug: !debug, selectedId: undefined });
  }

  function handleToggleGrid() {
    useStudioStore.setState({ grid: !grid });
  }

  function handlePreview() {
    router.push("/studio/preview");
  }

  return (
    <div className="flex justify-between items-center h-full px-4 py-2">
      <div className="w-full flex items-center space-x-4 text-lg">
        <Link href="/create">
          <a className="cursor-pointer transition text-outline hover:text-inherit p-1">
            <MdArrowBackIosNew />
          </a>
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

      <div className="w-full h-full flex justify-end items-center space-x-2">
        <div className="h-full aspect-square">
          <Tooltip text={`${grid ? "Hide" : "Show"} Grid`} placement="bottom">
            <IconButton selected={grid} onClick={handleToggleGrid}>
              <MdOutlineGridOn />
            </IconButton>
          </Tooltip>
        </div>

        <div className="h-full aspect-square">
          <Tooltip
            text={`${debug ? "Hide" : "Show"} Colliders`}
            placement="bottom"
          >
            <IconButton selected={debug} onClick={handleToggleDebug}>
              <HiCubeTransparent />
            </IconButton>
          </Tooltip>
        </div>

        <div className="h-full aspect-square">
          <Tooltip text="Preview" placement="bottom">
            <div className="h-full">
              <IconButton onClick={handlePreview}>
                <MdPreview />
              </IconButton>
            </div>
          </Tooltip>
        </div>

        <div className="pl-2">
          {handle ? <PublishButton /> : <LoginButton />}
        </div>
      </div>
    </div>
  );
}

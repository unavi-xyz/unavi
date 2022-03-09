import { MdArrowBackIosNew } from "react-icons/md";
import { useRouter } from "next/router";

import useLocalWorld from "../../../helpers/localWorlds/useLocalWorld";
import MiddleButtons from "./MiddleButtons";
import { mergeLocalWorld } from "../../../helpers/localWorlds/db";

interface Props {
  id: string;
}

export default function Navbar({ id }: Props) {
  const router = useRouter();

  const world = useLocalWorld(id);

  async function handleBack() {
    const canvas = document.querySelector("canvas");
    const image = canvas.toDataURL("image/jpeg", 0.5);

    await mergeLocalWorld(id, { image });

    router.push(`/editor/${id}`);
  }

  return (
    <div className="w-screen h-12 bg-white flex items-center justify-between px-2 border-b-[1px] border-neutral-200">
      <div className="flex items-center space-x-2 w-1/3">
        <div
          onClick={handleBack}
          className="hover:cursor-pointer text-xl p-2 rounded-full"
        >
          <MdArrowBackIosNew />
        </div>

        <div className="text-lg">{world?.name ?? id}</div>
      </div>

      <div className="w-1/3 flex justify-center text-2xl">
        <MiddleButtons />
      </div>

      <div className="w-1/3"></div>
    </div>
  );
}

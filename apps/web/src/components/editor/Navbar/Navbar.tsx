import { MdArrowBackIosNew } from "react-icons/md";
import Link from "next/link";

import useLocalWorld from "../../../helpers/localWorlds/useLocalWorld";
import MiddleButtons from "./MiddleButtons";

interface Props {
  id: string;
}

export default function Navbar({ id }: Props) {
  const world = useLocalWorld(id);

  return (
    <div className="w-screen h-12 bg-white flex items-center justify-between px-2 border-b-[1px] border-neutral-200">
      <div className="flex items-center space-x-2 w-1/3">
        <Link href={`/editor/${id}`} passHref>
          <div className="hover:cursor-pointer text-xl p-2 rounded-full">
            <MdArrowBackIosNew />
          </div>
        </Link>

        <div className="text-lg">{world?.name ?? id}</div>
      </div>

      <div className="w-1/3 flex justify-center text-2xl">
        <MiddleButtons />
      </div>

      <div className="w-1/3"></div>
    </div>
  );
}

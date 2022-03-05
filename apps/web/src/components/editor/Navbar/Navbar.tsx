import Link from "next/link";
import { MdArrowBackIosNew } from "react-icons/md";

import useLocalWorld from "../localWorlds/useLocalWorld";

interface Props {
  id: string;
}

export default function Navbar({ id }: Props) {
  const world = useLocalWorld(id);

  return (
    <div className="w-screen h-16 bg-white flex items-center px-8">
      <div className="flex items-center space-x-4">
        <Link href={`/editor/${id}`} passHref>
          <div className="hover:cursor-pointer text-lg p-2 rounded-full">
            <MdArrowBackIosNew />
          </div>
        </Link>

        <div className="text-xl">{world?.name ?? id}</div>
      </div>
    </div>
  );
}

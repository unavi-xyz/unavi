import { FaHammer } from "react-icons/fa";
import { IoMdSettings } from "react-icons/io";
import { MdCloudUpload, MdArrowBackIosNew } from "react-icons/md";
import { useRouter } from "next/router";

import useLocalWorld from "../../../components/editor/localWorlds/useLocalWorld";
import SidebarLayout from "../../../layouts/SidebarLayout/SidebarLayout";
import Link from "next/link";

export default function Id() {
  const router = useRouter();
  const id = router.query.id as string;

  const world = useLocalWorld(id);

  if (!world) {
    return <div className="p-16">World not found.</div>;
  }

  return (
    <div className="p-16 space-y-4">
      <div className="flex items-center space-x-12">
        <Link href="/editor" passHref>
          <div className="text-xl hover:cursor-pointer p-2 rounded-full">
            <MdArrowBackIosNew />
          </div>
        </Link>

        <div className="text-3xl">{world?.name ?? id}</div>

        <div className="flex items-center h-min bg-neutral-200 rounded">
          <Link href={`/editor/${id}/edit`} passHref>
            <div
              className="py-1.5 px-4 rounded-l hover:shadow hover:cursor-pointer
                       hover:bg-amber-300 text-xl"
            >
              <FaHammer />
            </div>
          </Link>
          <div
            className="py-1.5 px-4 hover:shadow hover:cursor-pointer
                     hover:bg-amber-300 text-xl"
          >
            <IoMdSettings />
          </div>
          <div
            className="py-1.5 px-4 rounded-r hover:shadow hover:cursor-pointer
                     hover:bg-amber-300 text-xl"
          >
            <MdCloudUpload />
          </div>
        </div>
      </div>
    </div>
  );
}

Id.Layout = SidebarLayout;

import { MdArrowBackIosNew } from "react-icons/md";
import { useRouter } from "next/router";
import Link from "next/link";

import { useLocalSpace } from "../../../helpers/indexeddb/localSpaces/useLocalScene";

import ObjectButton from "./ObjectButton";

export default function StudioNavbar() {
  const router = useRouter();
  const id = router.query.id as string;

  const localSpace = useLocalSpace(id);

  return (
    <div className="flex justify-between items-center h-full px-4">
      <div className="w-full flex items-center space-x-4 text-lg">
        <Link href={`/create/${id}`} passHref>
          <div className="cursor-pointer">
            <MdArrowBackIosNew />
          </div>
        </Link>

        <div>{localSpace?.name}</div>
      </div>

      <div className="w-full h-full flex justify-center items-center space-x-2 p-2">
        <ObjectButton />
      </div>

      <div className="w-full"></div>
    </div>
  );
}

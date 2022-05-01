import { useEffect } from "react";
import { MdArrowBackIosNew } from "react-icons/md";
import { useRouter } from "next/router";

import { useLocalSpace } from "../../../helpers/indexedDB/localSpaces/hooks/useLocalScene";

import ObjectButton from "./ObjectButton";
import { updateLocalSpace } from "../../../helpers/indexedDB/localSpaces/helpers";

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
        <ObjectButton />
      </div>

      <div className="w-full"></div>
    </div>
  );
}

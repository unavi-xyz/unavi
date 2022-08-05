import Link from "next/link";
import { useRouter } from "next/router";
import { MdClose } from "react-icons/md";

import MetaTags from "../../../src/ui/MetaTags";

export default function Preview() {
  const router = useRouter();
  const id = router.query.id;

  return (
    <>
      <MetaTags title="Preview" />

      <div className="h-full">
        <div className="crosshair" />

        <div onClick={(e) => e.stopPropagation()} className="fixed top-0 right-0 p-6 text-2xl">
          <Link href={`/studio/${id}`} passHref>
            <a
              className="block cursor-pointer p-2 rounded-full bg-surface text-onSurface
                         backdrop-blur bg-opacity-60 hover:bg-opacity-100 transition active:bg-opacity-90"
            >
              <MdClose />
            </a>
          </Link>
        </div>
      </div>
    </>
  );
}

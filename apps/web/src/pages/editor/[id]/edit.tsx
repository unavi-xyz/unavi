import { useRouter } from "next/router";

import Navbar from "../../../components/editor/Navbar/Navbar";
import EditorSidebar from "../../../components/editor/EditorSidebar/EditorSidebar";

export default function Id() {
  const router = useRouter();
  const id = router.query.id as string;

  return (
    <div className="h-full">
      <Navbar id={id} />

      <div className="flex h-full">
        <div className="h-full w-full bg-neutral-200"></div>
        <div className="h-full w-full max-w-md">
          <EditorSidebar />
        </div>
      </div>
    </div>
  );
}

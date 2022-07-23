import { CgSpinner } from "react-icons/cg";
import { AnimationAction } from "three";

import PanelPage from "./PanelPage";

interface Props {
  loaded: boolean;
  animations?: AnimationAction[];
}

export default function Panel({ loaded, animations }: Props) {
  return (
    <div className="absolute right-0 bottom-0 m-4">
      <div className="bg-white rounded-md w-96 h-72 p-4">
        {loaded ? (
          <PanelPage animations={animations} />
        ) : (
          <div className="h-full flex flex-col justify-center items-center">
            <CgSpinner className="animate-spin text-lg" />
          </div>
        )}
      </div>
    </div>
  );
}

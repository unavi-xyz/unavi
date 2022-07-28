import { CgSpinner } from "react-icons/cg";
import { AnimationAction } from "three";

import { RenderInfo, Settings } from "../ExampleCanvas";
import PanelPage from "./PanelPage";

interface Props {
  loaded: boolean;
  animations?: AnimationAction[];
  info?: RenderInfo;
  settings?: Settings;
  setSettings?: (settings: Settings) => void;
}

export default function Panel({ loaded, ...rest }: Props) {
  return (
    <div className="absolute right-0 bottom-0 m-4">
      <div className="bg-white rounded-md w-96 h-96 p-4">
        {loaded ? (
          <PanelPage {...rest} />
        ) : (
          <div className="h-full flex flex-col justify-center items-center">
            <CgSpinner className="animate-spin text-lg" />
          </div>
        )}
      </div>
    </div>
  );
}

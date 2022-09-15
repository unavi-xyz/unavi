import { CgSpinner } from "react-icons/cg";

import { RenderInfo, Settings } from "../ExampleCanvas";
import PanelPage from "./PanelPage";

interface Props {
  loaded: boolean;
  animations?: any[];
  info?: RenderInfo;
  settings?: Settings;
  setSettings?: (settings: Settings) => void;
}

export default function Panel({ loaded, ...rest }: Props) {
  return (
    <div className="absolute right-0 bottom-0 m-4">
      <div className="h-96 w-96 rounded-md bg-white p-4">
        {loaded ? (
          <PanelPage {...rest} />
        ) : (
          <div className="flex h-full flex-col items-center justify-center">
            <CgSpinner className="animate-spin text-lg" />
          </div>
        )}
      </div>
    </div>
  );
}

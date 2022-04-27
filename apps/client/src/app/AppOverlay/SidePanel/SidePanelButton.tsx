import { Dispatch } from "react";
import { SetStateAction, useAtom } from "jotai";

import { pageAtom } from "./helpers/atoms";
import { Page } from "./helpers/types";
import { Tooltip } from "../../../components/base";

interface Props {
  name: Page;
  setOpen: Dispatch<SetStateAction<boolean>>;
  icon: JSX.Element;
}

export default function SidePanelButton({ name, setOpen, icon }: Props) {
  const [page, setPage] = useAtom(pageAtom);

  return (
    <Tooltip text={name} placement="left">
      <div
        onClick={(e) => {
          e.stopPropagation();
          if (page === name) {
            setOpen((prev) => !prev);
          } else {
            setPage(name);
            setOpen(true);
          }
        }}
        className="bg-white rounded-lg flex items-center justify-center
                   cursor-pointer z-10 hover:bg-neutral-100 h-12 w-12 text-lg"
      >
        {icon}
      </div>
    </Tooltip>
  );
}

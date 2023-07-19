import { MdArrowBack } from "react-icons/md";

import { usePlayStore } from "@/app/play/playStore";
import { LeftPanelPage, RightPanelPage } from "@/app/play/types";

interface PropsBase {
  children: React.ReactNode;
  title: string;
  onBack?: () => void;
}

interface PropsNone extends PropsBase {
  side?: undefined;
  parentPage?: undefined;
}

interface PropsLeft extends PropsBase {
  side: "left";
  parentPage: LeftPanelPage;
}

interface PropsRight extends PropsBase {
  side: "right";
  parentPage: RightPanelPage;
}

type Props = PropsNone | PropsLeft | PropsRight;

export default function PanelPage({
  children,
  title,
  side,
  parentPage,
  onBack,
}: Props) {
  function handleBack() {
    if (onBack) {
      onBack();
    }

    if (!side) return;

    if (side === "left") {
      usePlayStore.setState({ leftPage: parentPage });
    } else {
      usePlayStore.setState({ rightPage: parentPage });
    }
  }

  const showBack = side !== undefined || onBack !== undefined;

  return (
    <div className="h-full space-y-4 overflow-hidden p-4">
      <div className="grid grid-cols-5">
        <div className="col-span-1">
          {showBack ? (
            <button
              onClick={handleBack}
              className="flex aspect-square h-full w-min items-center justify-center rounded-full text-xl transition hover:bg-white/10 active:opacity-80"
            >
              <MdArrowBack />
            </button>
          ) : null}
        </div>

        <h1
          title={title}
          className="col-span-3 overflow-hidden text-ellipsis text-center text-xl"
        >
          {title}
        </h1>
      </div>

      {children}
    </div>
  );
}

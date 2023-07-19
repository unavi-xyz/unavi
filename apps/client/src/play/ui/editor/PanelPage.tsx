import { MdArrowBack } from "react-icons/md";

import { usePlayStore } from "@/app/play/playStore";
import { LeftPanelPage, RightPanelPage } from "@/app/play/types";

interface PropsBase {
  children: React.ReactNode;
  title: string;
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
}: Props) {
  function handleBack() {
    if (!side) return;

    if (side === "left") {
      usePlayStore.setState({ leftPage: parentPage });
    } else {
      usePlayStore.setState({ rightPage: parentPage });
    }
  }

  return (
    <div className="space-y-4">
      <div className="grid grid-cols-3">
        {!side ? (
          <div></div>
        ) : (
          <button
            onClick={handleBack}
            className="aspect-square w-min rounded-full p-1 text-xl transition hover:bg-white/10 active:opacity-80"
          >
            <MdArrowBack />
          </button>
        )}

        <h1 className="whitespace-nowrap text-center text-xl">{title}</h1>
      </div>

      {children}
    </div>
  );
}

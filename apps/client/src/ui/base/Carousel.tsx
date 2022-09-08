import { IoMdArrowRoundBack, IoMdArrowRoundForward } from "react-icons/io";

import Button from "./Button";

interface Props {
  children: React.ReactNode;
  title?: string;
  disableBack?: boolean;
  disableForward?: boolean;
  onBack?: () => void;
  onForward?: () => void;
  height?: string;
}

export default function Carousel({
  children,
  title,
  disableBack,
  disableForward,
  onBack,
  onForward,
  height,
}: Props) {
  return (
    <div>
      <div className="text-2xl font-bold">{title}</div>
      <div className="flex space-x-4 py-4">
        <div className="flex items-center justify-center">
          <Button variant="tonal" icon disabled={disableBack} onClick={onBack}>
            <IoMdArrowRoundBack className="text-lg" />
          </Button>
        </div>

        <div
          className={`w-full p-2 overflow-x-hidden grid grid-flow-col gap-2 ${height}`}
        >
          {children}
        </div>

        <div className="flex items-center justify-center">
          <Button
            variant="tonal"
            icon
            disabled={disableForward}
            onClick={onForward}
          >
            <IoMdArrowRoundForward className="text-lg" />
          </Button>
        </div>
      </div>
    </div>
  );
}

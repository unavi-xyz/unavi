import { IoMdArrowRoundBack, IoMdArrowRoundForward } from "react-icons/io";

import Button from "./Button";

interface Props {
  children: React.ReactNode;
  title?: string;
  disableBack?: boolean;
  disableNext?: boolean;
  onBack?: () => void;
  onNext?: () => void;
  height?: string;
}

export default function Carousel({
  children,
  title,
  disableBack,
  disableNext,
  onBack,
  onNext,
  height,
}: Props) {
  return (
    <div>
      <div className="text-2xl font-bold">{title}</div>
      <div className="flex space-x-2">
        <div className="flex items-center justify-center">
          <Button variant="tonal" icon disabled={disableBack} onClick={onBack}>
            <IoMdArrowRoundBack className="text-lg" />
          </Button>
        </div>

        <div className={`grid w-full grid-flow-col gap-3 overflow-x-hidden pl-2 pt-2 ${height}`}>
          {children}
        </div>

        <div className="flex items-center justify-center">
          <Button variant="tonal" icon disabled={disableNext} onClick={onNext}>
            <IoMdArrowRoundForward className="text-lg" />
          </Button>
        </div>
      </div>
    </div>
  );
}

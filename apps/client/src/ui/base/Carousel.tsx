import { IoMdArrowRoundBack, IoMdArrowRoundForward } from "react-icons/io";

import Button from "./Button";

interface Props {
  children: React.ReactNode;
  title?: string;
  back?: boolean;
  forward?: boolean;
  onBack?: () => void;
  onForward?: () => void;
}

export default function Carousel({ children, title, back, forward, onBack, onForward }: Props) {
  return (
    <div>
      <div className="text-2xl font-bold">{title}</div>
      <div className="flex space-x-4 py-4">
        <div className="flex items-center justify-center">
          <Button variant="tonal" icon disabled={!back} onClick={onBack}>
            <IoMdArrowRoundBack className="text-lg" />
          </Button>
        </div>

        <div className="w-full p-2 overflow-x-hidden grid grid-flow-col">{children}</div>

        <div className="flex items-center justify-center">
          <Button variant="tonal" icon disabled={!forward} onClick={onForward}>
            <IoMdArrowRoundForward className="text-lg" />
          </Button>
        </div>
      </div>
    </div>
  );
}

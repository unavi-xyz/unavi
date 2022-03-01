import { Dispatch, ReactChild, SetStateAction } from "react";

interface Props {
  open: boolean;
  setOpen: Dispatch<SetStateAction<boolean>>;
  children: ReactChild | ReactChild[];
}
export default function Dialog({ open, setOpen, children }: Props) {
  if (!open) return null;

  return (
    <div
      className="fixed top-0 left-0 z-10 bg-black bg-opacity-50 w-screen
                 h-screen flex flex-col justify-center"
      onClick={() => setOpen(false)}
    >
      <dialog
        open
        onClick={(e) => e.stopPropagation()}
        className="fade mx-auto w-full max-w-lg flex flex-col space-y-4 p-8 rounded-lg"
      >
        {children}
      </dialog>
    </div>
  );
}

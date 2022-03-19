import { MdArrowForwardIos } from "react-icons/md";

interface Props {
  name?: string;
  [key: string]: any;
}

export default function PackButton({ name, ...rest }: Props) {
  return (
    <div
      {...rest}
      className="flex items-center justify-between px-8 border h-20 text-xl
               hover:bg-neutral-100 hover:cursor-pointer rounded-full"
    >
      {name}
      <MdArrowForwardIos />
    </div>
  );
}

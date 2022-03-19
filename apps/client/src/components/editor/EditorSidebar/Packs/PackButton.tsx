import { MdArrowForwardIos } from "react-icons/md";

interface Props {
  name?: string;
  icon?: JSX.Element;
  [key: string]: any;
}

export default function PackButton({ name, icon, ...rest }: Props) {
  return (
    <div
      {...rest}
      className="group flex w-full space-x-2 p-1 bg-neutral-100
               hover:bg-neutral-200 hover:cursor-pointer rounded-lg h-20"
    >
      <div className="flex items-center text-3xl px-4 text-neutral-500">
        {icon}
      </div>

      <div className="flex w-full items-center justify-between">
        <h3 className="text-lg">{name}</h3>

        <div className="text-lg pr-6">
          <MdArrowForwardIos />
        </div>
      </div>
    </div>
  );
}

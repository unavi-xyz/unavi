import Link from "next/link";
import { useRouter } from "next/router";

export enum Colors {
  sky = "group-hover:bg-sky-400",
  lime = "group-hover:bg-lime-400",
  amber = "group-hover:bg-amber-400",
  red = "group-hover:bg-red-400",
}

interface Props {
  icon: JSX.Element;
  text: string;
  href?: string;
  color?: Colors;
  [key: string]: any;
}

export default function SidebarButton({
  icon,
  text,
  href,
  color = Colors.sky,
  ...rest
}: Props) {
  const router = useRouter();
  const selected = router.asPath === href;

  const button = (
    <button
      {...rest}
      className="group flex items-center space-x-3 p-2 rounded-lg hover:text-black
           hover:cursor-pointer text-neutral-500"
    >
      <div
        className={`w-7 h-7 flex items-center justify-center bg-neutral-200 rounded-md
   ${color} ${
          selected
            ? color === Colors.sky
              ? "bg-sky-400"
              : color === Colors.lime
              ? "bg-lime-400"
              : color === Colors.amber
              ? "bg-amber-400"
              : color === Colors.red
              ? "bg-red-400"
              : null
            : null
        }`}
        style={{ color: selected ? "black" : null }}
      >
        {icon}
      </div>
      <div style={{ color: selected ? "black" : null }}>{text}</div>
    </button>
  );

  if (href) {
    return (
      <Link href={href} passHref>
        {button}
      </Link>
    );
  }
  return button;
}

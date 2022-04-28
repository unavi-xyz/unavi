import Link from "next/link";
import { useRouter } from "next/router";

interface Props {
  href: string;
  text: string;
}

export default function NavbarTextButton({ href, text }: Props) {
  const router = useRouter();

  const bgColor = router.pathname === href ? "bg-neutral-200" : "e";

  return (
    <Link href={href} passHref>
      <button
        className={`cursor-pointer hover:bg-neutral-200 rounded-md px-2 py-1 text-sm font-bold ${bgColor}`}
      >
        {text}
      </button>
    </Link>
  );
}

import Link from "next/link";
import { useRouter } from "next/router";

interface Props {
  children: React.ReactNode;
  href: string;
}

export default function SidebarButton({ children, href }: Props) {
  const router = useRouter();
  const path = router.pathname;

  const selected = path === href ? "bg-neutral-200" : "hover:bg-neutral-200";

  return (
    <Link href={href}>
      <div
        className={`cursor-pointer rounded px-4 text-lg transition ${selected}`}
      >
        {children}
      </div>
    </Link>
  );
}

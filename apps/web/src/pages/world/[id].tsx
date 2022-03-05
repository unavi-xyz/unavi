import { useRouter } from "next/router";

export default function World() {
  const router = useRouter();
  const id = router.query.id as string;

  return <div>{id}</div>;
}

import { useRouter } from "next/router";

import NavbarLayout from "../../src/components/layouts/NavbarLayout/NavbarLayout";

export default function Space() {
  const router = useRouter();
  const id = router.query.id as string;

  return <div>Space {id}</div>;
}

Space.Layout = NavbarLayout;

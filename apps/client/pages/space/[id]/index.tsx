import Head from "next/head";
import { useRouter } from "next/router";

import SpaceLayout from "../../../src/components/layouts/SpaceLayout/SpaceLayout";
import { usePublication } from "../../../src/helpers/lens/hooks/usePublication";

export default function Space() {
  const router = useRouter();
  const id = router.query.id as string;

  const publication = usePublication(id);

  return (
    <div className="space-y-2">
      <div className="text-2xl font-bold">Description</div>
      <div className="text-lg text-outline">
        {publication?.metadata.description}
      </div>
    </div>
  );
}

Space.Layout = SpaceLayout;

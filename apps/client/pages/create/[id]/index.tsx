import { useRouter } from "next/router";

import { getLocalSpaceLayout } from "../../../src/components/layouts/LocalSpaceLayout/LocalSpaceLayout";
import { useLocalSpace } from "../../../src/helpers/indexeddb/LocalSpace/hooks/useLocalSpace";

export default function Id() {
  const router = useRouter();
  const id = router.query.id as string;

  const localSpace = useLocalSpace(id);

  if (!localSpace) return null;

  return (
    <div className="space-y-2">
      <div className="text-2xl font-bold">About</div>
      <div className="text-lg text-outline">{localSpace.description}</div>
    </div>
  );
}

Id.getLayout = getLocalSpaceLayout;

import { WorldMetadata } from "@wired-protocol/types";
import { eq } from "drizzle-orm";

import { getSession } from "@/src/server/auth/getSession";
import { db } from "@/src/server/db/drizzle";
import { world } from "@/src/server/db/schema";
import ButtonTabs, { TabContent } from "@/src/ui/ButtonTabs";
import { WorldId } from "@/src/utils/parseWorldId";

import About from "./About";
import Settings from "./Settings";

interface Props {
  id: WorldId;
  metadata: WorldMetadata;
}

export default async function Tabs({ id, metadata }: Props) {
  const session = await getSession();

  const owner = await fetchWorldOwner(id.value);

  const isOwner = session?.user.userId === owner;

  return (
    <>
      {isOwner ? (
        <ButtonTabs titles={["About", "Settings"]}>
          <TabContent value="About">
            <About metadata={metadata} />
          </TabContent>
          <TabContent value="Settings">
            <Settings id={id} />
          </TabContent>
        </ButtonTabs>
      ) : (
        <About metadata={metadata} />
      )}
    </>
  );
}

async function fetchWorldOwner(id: string) {
  const found = await db.query.world.findFirst({
    columns: { ownerId: true },
    where: eq(world.publicId, id),
  });

  return found?.ownerId;
}

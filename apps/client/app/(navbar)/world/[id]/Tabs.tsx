import { WorldMetadata } from "@wired-protocol/types";
import { eq } from "drizzle-orm";

import { getUserSession } from "@/src/server/auth/getUserSession";
import { db } from "@/src/server/db/drizzle";
import { world } from "@/src/server/db/schema";
import ButtonTabs, { TabContent } from "@/src/ui/ButtonTabs";
import { SpaceId } from "@/src/utils/parseSpaceId";

import About from "./About";
import Settings from "./Settings";

interface Props {
  id: SpaceId;
  metadata: WorldMetadata;
}

export default async function Tabs({ id, metadata }: Props) {
  const session = await getUserSession();

  const owner = await fetchSpaceDBOwner(id.value);

  const isOwner = session?.user.userId === owner;

  return (
    <>
      {isOwner ? (
        <ButtonTabs titles={["About", "Settings"]}>
          <TabContent value="About">
            <About id={id} metadata={metadata} />
          </TabContent>
          <TabContent value="Settings">
            <Settings id={id} metadata={metadata} />
          </TabContent>
        </ButtonTabs>
      ) : (
        <About id={id} metadata={metadata} />
      )}
    </>
  );
}

async function fetchSpaceDBOwner(id: string) {
  const found = await db.query.world.findFirst({
    columns: { ownerId: true },
    where: eq(world.publicId, id),
  });

  return found?.ownerId;
}

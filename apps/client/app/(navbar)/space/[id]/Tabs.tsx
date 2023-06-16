import { WorldMetadata } from "@wired-protocol/types";

import { getUserSession } from "@/src/server/auth/getUserSession";
import { fetchNFTSpaceOwner } from "@/src/server/helpers/fetchNFTSpaceOwner";
import { prisma } from "@/src/server/prisma";
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

  const owner =
    id.type === "tokenId"
      ? await fetchNFTSpaceOwner(id.value)
      : await fetchSpaceDBOwner(id.value);

  const isOwner =
    id.type === "tokenId"
      ? session?.user.address === owner
      : session?.user.userId === owner;

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
  const space = await prisma.space.findFirst({
    select: { ownerId: true },
    where: { publicId: id },
  });

  return space?.ownerId;
}

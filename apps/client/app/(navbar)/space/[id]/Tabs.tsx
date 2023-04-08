import { fetchSpaceNFTOwner } from "@/src/server/helpers/fetchSpaceNFTOwner";
import { getServerSession } from "@/src/server/helpers/getServerSession";
import { SpaceMetadata } from "@/src/server/helpers/readSpaceMetadata";
import ButtonTabs, { TabContent } from "@/src/ui/ButtonTabs";
import { SpaceId } from "@/src/utils/parseSpaceId";

import About from "./About";
import Settings from "./Settings";

interface Props {
  id: SpaceId;
  metadata: SpaceMetadata;
}

export default async function Tabs({ id, metadata }: Props) {
  const session = await getServerSession();

  const owner = id.type === "tokenId" ? await fetchSpaceNFTOwner(id.value) : null;

  const isOwner = owner && session?.address === owner;

  return (
    <>
      {isOwner ? (
        <ButtonTabs titles={["About", "Settings"]}>
          <TabContent value="About">
            {/* @ts-expect-error Server Component */}
            <About description={metadata.description} />
          </TabContent>
          <TabContent value="Settings">
            {/* @ts-expect-error Server Component */}
            <Settings id={id} metadata={metadata} />
          </TabContent>
        </ButtonTabs>
      ) : (
        // @ts-expect-error Server Component
        <About description={metadata.description} />
      )}
    </>
  );
}

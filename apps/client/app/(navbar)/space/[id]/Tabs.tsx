import { fetchSpaceOwner } from "../../../../src/server/helpers/fetchSpaceOwner";
import { getServerSession } from "../../../../src/server/helpers/getServerSession";
import ButtonTabs, { TabContent } from "../../../../src/ui/ButtonTabs";
import About from "./About";
import Settings from "./Settings";

interface Props {
  id: number;
}

export default async function Tabs({ id }: Props) {
  const [session, owner] = await Promise.all([getServerSession(), fetchSpaceOwner(id)]);

  const isOwner = session?.address === owner;

  return (
    <>
      {isOwner ? (
        <ButtonTabs titles={["About", "Settings"]}>
          <TabContent value="About">
            {/* @ts-expect-error Server Component */}
            <About id={id} />
          </TabContent>
          <TabContent value="Settings">
            {/* @ts-expect-error Server Component */}
            <Settings id={id} />
          </TabContent>
        </ButtonTabs>
      ) : (
        // @ts-expect-error Server Component
        <About id={id} />
      )}
    </>
  );
}

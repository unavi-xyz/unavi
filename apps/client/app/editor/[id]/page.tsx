import RainbowkitWrapper from "../../(navbar)/RainbowkitWrapper";
import SessionProvider from "../../(navbar)/SessionProvider";
import Editor from "./Editor";

type Params = {
  id: string;
};

export default function Page({ params: { id } }: { params: Params }) {
  return (
    <SessionProvider>
      <RainbowkitWrapper>
        <Editor id={id} />
      </RainbowkitWrapper>
    </SessionProvider>
  );
}

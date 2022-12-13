import { GetServerSidePropsContext, InferGetServerSidePropsType } from "next";
import { useRouter } from "next/router";

import { getPublicationProps } from "../../../client/lens/utils/getPublicationProps";
import { trpc } from "../../../client/trpc";
import { getNavbarLayout } from "../../../home/layouts/NavbarLayout/NavbarLayout";
import SpaceLayout from "../../../home/layouts/SpaceLayout/SpaceLayout";
import { bytesToDisplay } from "../../../utils/bytesToDisplay";
import { numberToCommas } from "../../../utils/numberToCommas";

export const getServerSideProps = async ({ res, query }: GetServerSidePropsContext) => {
  const ONE_MINUTE_IN_SECONDS = 60;
  const ONE_WEEK_IN_SECONDS = 60 * 60 * 24 * 7;

  res.setHeader(
    "Cache-Control",
    `public, s-maxage=${ONE_MINUTE_IN_SECONDS}, stale-while-revalidate=${ONE_WEEK_IN_SECONDS}`
  );

  const id = query.id as string;
  const props = await getPublicationProps(id);

  return { props };
};

export default function Space(props: InferGetServerSidePropsType<typeof getServerSideProps>) {
  const router = useRouter();
  const id = router.query.id as string;

  const { data: stats } = trpc.public.modelStats.useQuery(
    { publicationId: id },
    {
      refetchOnWindowFocus: false,
      refetchOnMount: false,
      refetchOnReconnect: false,
    }
  );

  return (
    <SpaceLayout {...props}>
      <div className="space-y-4 text-lg">
        {props.publication?.metadata.description && (
          <div className="whitespace-pre-line">{props.publication?.metadata.description}</div>
        )}

        <div className="flex text-neutral-500">
          <div>
            {["File Size", "Polygons", "Materials", "Meshes", "Skins", "Bones"].map((title) => (
              <div key={title}>{title}</div>
            ))}
          </div>

          <div className="pl-4">
            {(stats
              ? [
                  bytesToDisplay(stats.fileSize),
                  numberToCommas(stats.polygonCount),
                  numberToCommas(stats.materialCount),
                  numberToCommas(stats.meshCount),
                  numberToCommas(stats.skinCount),
                  numberToCommas(stats.boneCount),
                ]
              : ["...", "...", "...", "...", "...", "..."]
            ).map((stat, i) => (
              <div key={i}>{stat}</div>
            ))}
          </div>
        </div>
      </div>
    </SpaceLayout>
  );
}

Space.getLayout = getNavbarLayout;

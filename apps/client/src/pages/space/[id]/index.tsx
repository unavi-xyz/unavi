import { GetServerSidePropsContext, InferGetServerSidePropsType } from "next";

import { getPublicationProps } from "../../../client/lens/utils/getPublicationProps";
import { getNavbarLayout } from "../../../home/layouts/NavbarLayout/NavbarLayout";
import SpaceLayout from "../../../home/layouts/SpaceLayout/SpaceLayout";
import { getGltfStats } from "../../../server/helpers/getGltfStats";
import { bytesToDisplay } from "../../../utils/bytesToDisplay";
import { numberToCommas } from "../../../utils/numberToCommas";

export const getServerSideProps = async ({
  res,
  query,
}: GetServerSidePropsContext) => {
  const ONE_HOUR_IN_SECONDS = 60 * 60;
  const ONE_WEEK_IN_SECONDS = ONE_HOUR_IN_SECONDS * 24 * 7;

  res.setHeader(
    "Cache-Control",
    `public, s-maxage=${ONE_HOUR_IN_SECONDS}, stale-while-revalidate=${ONE_WEEK_IN_SECONDS}`
  );

  const id = query.id as string;

  const publicationPropsPromise = getPublicationProps(id);
  const statsPromise = getGltfStats(id);

  const publicationProps = await publicationPropsPromise;
  const stats = await statsPromise;

  return {
    props: {
      ...publicationProps,
      stats,
    },
  };
};

export default function Space(
  props: InferGetServerSidePropsType<typeof getServerSideProps>
) {
  return (
    <SpaceLayout {...props}>
      <div className="space-y-8">
        {props.publication?.metadata.description && (
          <div className="space-y-2">
            <div className="text-2xl font-bold">Description</div>
            <div className="whitespace-pre-line text-lg text-neutral-500">
              {props.publication?.metadata.description}
            </div>
          </div>
        )}

        <div className="space-y-2">
          <div className="text-2xl font-bold">Stats</div>

          <div className="flex text-lg">
            <div>
              {[
                "File Size",
                "Polygons",
                "Materials",
                "Meshes",
                "Skins",
                "Bones",
              ].map((title) => (
                <div key={title} className="font-bold">
                  {title}
                </div>
              ))}
            </div>

            <div className="pl-8">
              {[
                bytesToDisplay(props.stats.fileSize),
                numberToCommas(props.stats.polygonCount),
                numberToCommas(props.stats.materialCount),
                numberToCommas(props.stats.meshCount),
                numberToCommas(props.stats.skinCount),
                numberToCommas(props.stats.boneCount),
              ].map((stat, i) => (
                <div key={i} className="text-neutral-500">
                  {stat}
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </SpaceLayout>
  );
}

Space.getLayout = getNavbarLayout;

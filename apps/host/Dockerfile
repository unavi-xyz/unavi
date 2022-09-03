FROM node:16 as base

ARG SCOPE
ENV SCOPE=${SCOPE}
ENV YARN_CACHE_FOLDER=.yarn-cache

# install mediasoup dependencies
RUN \
	set -x \
	&& apt-get update \
	&& apt-get install -y net-tools build-essential python3 python3-pip valgrind

FROM base AS pruner
# turborepo combines everything into a single lock file
# we need to run turbo prune to remove all deps we dont need
RUN yarn global add turbo
COPY . .
RUN turbo prune --scope=${SCOPE} --docker

FROM base AS dev-deps
COPY --from=pruner /out/json/ .
COPY --from=pruner /out/yarn.lock ./yarn.lock
RUN yarn install --frozen-lockfile

FROM base AS prod-deps
COPY --from=pruner /out/json/ .
COPY --from=pruner /out/yarn.lock ./yarn.lock
COPY --from=dev-deps /${YARN_CACHE_FOLDER} /${YARN_CACHE_FOLDER}
RUN yarn install --frozen-lockfile --production --prefer-offline --ignore-scripts
RUN rm -rf /${YARN_CACHE_FOLDER}

FROM base AS builder
COPY --from=dev-deps . .
COPY --from=pruner /out/full/ .
RUN yarn turbo run build --filter=${SCOPE}...
RUN find . -name node_modules | xargs rm -rf

FROM base AS runner
COPY --from=dev-deps . .
COPY --from=prod-deps . .
COPY --from=builder . .

ARG RTC_MIN_PORT
ENV RTC_MIN_PORT=${RTC_MIN_PORT}

ARG RTC_MAX_PORT
ENV RTC_MAX_PORT=${RTC_MAX_PORT}

CMD yarn workspace ${SCOPE} start
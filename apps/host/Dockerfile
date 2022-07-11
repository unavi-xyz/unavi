# turborepo combines everything into a single lock file
# we need to run turbo prune to remove all deps we dont need

FROM node:16 as base
WORKDIR /app

ARG SCOPE
ENV SCOPE=${SCOPE}
ENV YARN_CACHE_FOLDER=.yarn-cache

# install mediasoup dependencies
RUN \
	set -x \
	&& apt-get update \
	&& apt-get install -y net-tools build-essential python3 python3-pip valgrind

FROM base AS pruner
RUN yarn global add turbo
COPY . .
RUN turbo prune --scope=${SCOPE} --docker

FROM base AS dev-deps
COPY --from=pruner /app/out/json/ .
COPY --from=pruner /app/out/yarn.lock ./yarn.lock
RUN yarn install --frozen-lockfile

FROM base AS prod-deps
COPY --from=pruner /app/out/json/ .
COPY --from=pruner /app/out/yarn.lock ./yarn.lock
COPY --from=dev-deps /app/${YARN_CACHE_FOLDER} /${YARN_CACHE_FOLDER}
RUN yarn install --frozen-lockfile --production --prefer-offline --ignore-scripts
RUN rm -rf /app/${YARN_CACHE_FOLDER}

FROM base AS builder
COPY --from=dev-deps /app/ .
COPY --from=pruner /app/out/full/ .
RUN yarn turbo run build --filter=${SCOPE}...
RUN find . -name node_modules | xargs rm -rf

FROM base AS runner
COPY --from=prod-deps /app/ .
COPY --from=builder /app/ .

ARG RTC_MIN_PORT
ENV RTC_MIN_PORT=${RTC_MIN_PORT}

ARG RTC_MAX_PORT
ENV RTC_MAX_PORT=${RTC_MAX_PORT}

CMD [ "yarn", "workspace", ${SCOPE}, "start" ]
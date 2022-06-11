FROM elixir:alpine AS base
COPY . .
RUN mix local.hex --force
RUN mix local.rebar --force
RUN mix deps.get

FROM base as runner
RUN mix compile
CMD ["mix", "run", "--no-halt"]

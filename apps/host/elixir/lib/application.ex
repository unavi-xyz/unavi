defmodule Host.Application do
  use Application

  def start(_type, _args) do
    IO.puts("Host.Application.start")

    # Create player count table
    :ets.new(:player_count, [:named_table, :set, :public])

    children = [
      Plug.Cowboy.child_spec(
        scheme: :http,
        plug: Host.Router,
        options: [
          dispatch: dispatch(),
          port: 4000
        ]
      ),
      Registry.child_spec(
        keys: :duplicate,
        name: Registry.Host
      )
    ]

    opts = [strategy: :one_for_one, name: Host.Supervisor]
    Supervisor.start_link(children, opts)
  end

  defp dispatch do
    [
      {:_,
       [
         {"/ws/:id", Host.SocketHandler, []},
         {:_, Plug.Cowboy.Handler, {Host.Router, []}}
       ]}
    ]
  end
end

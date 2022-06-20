defmodule Host.SocketHandler do
  @behaviour :cowboy_websocket

  def init(request, _state) do
    state = %{space_id: request.path, user_id: UUID.uuid4(), handle: nil}
    {:cowboy_websocket, request, state}
  end

  # terminate if no activity for one minute
  @timeout 60000

  # Called on WebSocket initialization
  def websocket_init(state) do
    # Add the user to the space
    Registry.Host
    |> Registry.register(state.space_id, {})

    {:ok, state}
  end

  # Handle ping
  def websocket_handle({:text, "ping"}, req, state) do
    {:reply, {:text, "pong"}, req, state}
  end

  # Handle messages
  def websocket_handle({:text, json}, state) do
    payload = Jason.decode!(json)
    type = payload["type"]
    data = payload["data"]

    case type do
      # Set identity
      "identity" ->
        IO.puts("identity from #{state.user_id}")

        state = %{state | handle: data}

        {:ok, state}

      # Send a chat message
      "chatmessage" ->
        IO.puts("chatmessage from #{state.user_id}")

        username = state.handle || get_guest_name(state)

        chatMessage = %{
          type: "chatmessage",
          data: %{
            id: UUID.uuid4(),
            timestamp: :os.system_time(),
            username: username,
            message: data
          }
        }

        text = Jason.encode!(chatMessage)

        # Broadcast to all connections in the space
        Registry.Host
        |> Registry.dispatch(state.space_id, fn entries ->
          for {pid, _} <- entries do
            Process.send(pid, {:text, text}, [])
          end
        end)

        {:ok, state}

      # Send location
      "location" ->
        locationMessage = %{
          type: "location",
          data: %{
            userid: state.user_id,
            handle: state.handle,
            location: data
          }
        }

        text = Jason.encode!(locationMessage)

        # Broadcast to all other connections in the space
        Registry.Host
        |> Registry.dispatch(state.space_id, fn entries ->
          for {pid, _} <- entries do
            if pid != self() do
              Process.send(pid, {:text, text}, [])
            end
          end
        end)

        {:ok, state}

      _ ->
        {:error, "unknown type"}
    end
  end

  # Forward messages to the client
  def websocket_info(info, state) do
    {:reply, info, state}
  end

  # Called on WebSocket close
  def terminate(_reason, _req, state) do
    IO.puts("Terminating connection with #{state.user_id}")

    # Send a leave message to the space
    leaveMessage = %{
      type: "leave",
      data: %{
        userid: state.user_id
      }
    }

    text = Jason.encode!(leaveMessage)

    # Broadcast to all connections in the space
    Registry.Host
    |> Registry.dispatch(state.space_id, fn entries ->
      for {pid, _} <- entries do
        Process.send(pid, {:text, text}, [])
      end
    end)

    :ok
  end

  # Creates a guest username from their user ID
  defp get_guest_name(state) do
    <<head::binary-size(6)>> <> _ = state.user_id
    "Guest-#{head}"
  end
end

defmodule Host.SocketHandler do
  @behaviour :cowboy_websocket

  def init(request, _state) do
    state = %{space_id: request.bindings.id, user_id: UUID.uuid4(), handle: nil}
    {:cowboy_websocket, request, state}
  end

  # terminate if no activity for one minute
  @timeout 60000

  def websocket_init(state) do
    IO.puts("Initializing connection with #{state.user_id}")

    # Add user to the space
    Registry.Host
    |> Registry.register(state.space_id, {})

    # Initialize player count if it doesn't exist
    :ets.insert_new(:player_count, {state.space_id, 0})

    # Increase player count
    :ets.update_counter(:player_count, state.space_id, 1)

    {:ok, state}
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

    # Decrease player count
    :ets.update_counter(:player_count, state.space_id, -1)

    # Delete space player count if there are no more players
    [{_, count}] = :ets.lookup(:player_count, state.space_id)

    if count == 0 do
      :ets.delete(:player_count, state.space_id)
    end

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

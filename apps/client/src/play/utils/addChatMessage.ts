import { usePlayStore } from "../store";
import { ChatMessage } from "../ui/ChatMessage";

export function addChatMessage(message: ChatMessage) {
  const { chatMessages } = usePlayStore.getState();

  usePlayStore.setState({
    chatMessages: [...chatMessages, message]
      .sort((a, b) => b.timestamp - a.timestamp)
      .slice(0, 100),
  });
}

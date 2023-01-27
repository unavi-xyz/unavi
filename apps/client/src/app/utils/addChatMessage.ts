import { useAppStore } from "../store";
import { ChatMessage } from "../ui/ChatMessage";

export function addChatMessage(message: ChatMessage) {
  const { chatMessages } = useAppStore.getState();

  useAppStore.setState({
    chatMessages: [...chatMessages, message]
      .sort((a, b) => b.timestamp - a.timestamp)
      .slice(0, 100),
  });
}

export default function Chat() {
  return null;
  // const chatInputRef = useRef<HTMLInputElement>(null);
  // const focusedRef = useRef(false);

  // const messages = useAppStore((state) => state.messages);
  // const { socket } = useContext(NetworkingContext);

  // usePublishIdentity();

  // useEffect(() => {
  //   if (!socket) return;

  //   function onMessage(event: MessageEvent) {
  //     const { type, data } = JSON.parse(event.data) as RecievedWebsocketMessage;

  //     if (type === "chatmessage") {
  //       useAppStore.getState().addMessage(data);
  //     }
  //   }

  //   socket.on("recieve_chat_message", async (data) => {
  //     try {
  //       const message = ReceiveChatMessageDataSchema.parse(data);

  //       useAppStore.getState().addMessage(message);
  //     } catch (e) {
  //       console.error(e);
  //     }
  //   });
  // }, [socket]);

  // useEffect(() => {
  //   useAppStore.setState({ chatInputRef });
  // }, []);

  // function handleKeyDown(e: KeyboardEvent<HTMLInputElement>) {
  //   if (focusedRef.current !== true) return;

  //   //disable controls when typing
  //   e.stopPropagation();

  //   if (e.key === "Enter") {
  //     const target = e.target as HTMLInputElement;
  //     const text = target.value;

  //     if (text === "") return;

  //     const message = text;

  //     if (socket) {
  //       socket.emit("send_chat_message", { message }, (res) => {
  //         const { success } = SendChatMessageResponseSchema.parse(res);
  //         if (!success) {
  //           console.error("Failed to send chat message");
  //         }
  //       });
  //     }

  //     target.value = "";
  //   }
  // }

  // return (
  //   <div className="absolute bottom-0 p-8 w-full space-y-4 z-10">
  //     <div className="w-full max-w-lg overflow-y-hidden flex flex-col-reverse">
  //       {messages.map((message) => {
  //         return <ChatMessage key={message.messageId} message={message} />;
  //       })}
  //     </div>

  //     <input
  //       ref={chatInputRef}
  //       onKeyDown={handleKeyDown}
  //       onFocus={() => (focusedRef.current = true)}
  //       onBlur={() => (focusedRef.current = false)}
  //       type="text"
  //       maxLength={420}
  //       placeholder="Press T to chat"
  //       onClick={(e) => e.stopPropagation()}
  //       className="px-3 py-2 rounded outline-none w-full max-w-lg bg-surfaceDark
  //                  bg-opacity-50 backdrop-blur-xl focus:bg-opacity-100 text-sm transition
  //                text-onSurfaceDark placeholder-onSurfaceDark placeholder-opacity-60"
  //     />
  //   </div>
  // );
}

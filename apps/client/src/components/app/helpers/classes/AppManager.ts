import { UseBoundStore, StoreApi } from "zustand";
import { AppStore } from "../store";

type useStoreType = UseBoundStore<AppStore, StoreApi<AppStore>>;

export class AppManager {
  useStore: useStoreType;

  constructor(useStore: useStoreType) {
    this.useStore = useStore;
  }

  setChatInputRef(chatInputRef: AppStore["chatInputRef"]) {
    this.useStore.setState({ chatInputRef });
  }
}

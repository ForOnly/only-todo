import { reactive } from "vue";

export type ConfirmOptions = {
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  danger?: boolean;
};

type ConfirmState = {
  open: boolean;
  title: string;
  message: string;
  confirmLabel: string;
  cancelLabel: string;
  danger: boolean;
};

const state = reactive<ConfirmState>({
  open: false,
  title: "",
  message: "",
  confirmLabel: "",
  cancelLabel: "",
  danger: false,
});

let resolver: ((value: boolean) => void) | null = null;

/** 应用内确认框（Promise）；同时只允许一个进行中 */
export function confirm(options: ConfirmOptions): Promise<boolean> {
  if (resolver) {
    resolver(false);
    resolver = null;
  }
  state.title = options.title;
  state.message = options.message;
  state.confirmLabel = options.confirmLabel ?? "";
  state.cancelLabel = options.cancelLabel ?? "";
  state.danger = options.danger ?? false;
  state.open = true;
  return new Promise((resolve) => {
    resolver = resolve;
  });
}

export function resolveConfirm(value: boolean): void {
  state.open = false;
  const r = resolver;
  resolver = null;
  r?.(value);
}

export function useAppConfirmState() {
  return state;
}

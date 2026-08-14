/** 打开中的 AppModal Esc 处理器，后进先出，只让最顶层响应 */

type EscHandler = () => void;

const stack: EscHandler[] = [];
let listening = false;

function onWindowKeydown(event: KeyboardEvent) {
  if (event.key !== "Escape") return;
  const top = stack[stack.length - 1];
  if (!top) return;
  top();
}

function ensureListen() {
  if (listening) return;
  window.addEventListener("keydown", onWindowKeydown);
  listening = true;
}

function stopListen() {
  if (!listening) return;
  window.removeEventListener("keydown", onWindowKeydown);
  listening = false;
}

/** 弹窗打开时登记；关闭/卸载时调用返回的函数 */
export function registerModalEsc(handler: EscHandler): () => void {
  stack.push(handler);
  ensureListen();
  return () => {
    const index = stack.lastIndexOf(handler);
    if (index >= 0) stack.splice(index, 1);
    if (stack.length === 0) stopListen();
  };
}

/** 有模态打开时，主窗全局快捷键应让出 Esc */
export function modalEscDepth(): number {
  return stack.length;
}

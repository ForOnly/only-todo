import { openContextMenu, closeContextMenu } from "@/composables/useAppContextMenu";

/** 三窗共用：禁用系统右键/文件拖入，并打开应用内菜单 */
export function installDesktopGuards(): void {
  document.addEventListener(
    "contextmenu",
    (event) => {
      event.preventDefault();
      closeContextMenu();
      openContextMenu(event);
    },
    true,
  );

  document.addEventListener(
    "dragover",
    (event) => {
      event.preventDefault();
    },
    true,
  );

  document.addEventListener(
    "drop",
    (event) => {
      event.preventDefault();
    },
    true,
  );
}

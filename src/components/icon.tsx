import {
  Add01Icon,
  ArrowDataTransferHorizontalIcon,
  ArrowDown01Icon,
  ArrowExpand01Icon,
  ArrowUp01Icon,
  ArrowUpIcon,
  Cancel01Icon,
  Chatting01Icon,
  Copy01Icon,
  DragDropVerticalIcon,
  LanguageCircleIcon,
  Menu02Icon,
  MessageQuestionIcon,
  Pen01Icon,
  Pin02Icon,
  Settings01Icon,
  StopIcon,
  Tick02Icon,
  VolumeHighIcon,
  VolumeLowIcon,
  VolumeOffIcon,
} from "@hugeicons/core-free-icons";
import { HugeiconsIcon, type IconSvgElement } from "@hugeicons/react";

export type Icon = React.ComponentType<React.ComponentProps<"svg"> & { strokeWidth?: number }>;

function createHugeicon(icon: IconSvgElement, defaultStrokeWidth = 2): Icon {
  const Component = ({ strokeWidth, ...props }: React.ComponentProps<"svg"> & { strokeWidth?: number }) => (
    <HugeiconsIcon icon={icon} color="currentColor" strokeWidth={strokeWidth ?? defaultStrokeWidth} {...props} />
  );
  return Component;
}

const _Icons = {
  gripVertical: createHugeicon(DragDropVerticalIcon),
  arrowExpand: createHugeicon(ArrowExpand01Icon),
  arrowUp: createHugeicon(ArrowUpIcon),
  pin: createHugeicon(Pin02Icon),
  add: createHugeicon(Add01Icon),
  x: createHugeicon(Cancel01Icon),
  tick: createHugeicon(Tick02Icon),
  copy: createHugeicon(Copy01Icon),
  volumeLow: createHugeicon(VolumeLowIcon),
  volumeOff: createHugeicon(VolumeOffIcon),
  volumeHigh: createHugeicon(VolumeHighIcon),
  list: createHugeicon(Menu02Icon),
  pen: createHugeicon(Pen01Icon),
  settings: createHugeicon(Settings01Icon),
  languages: createHugeicon(LanguageCircleIcon),
  stop: createHugeicon(StopIcon),
  chat: createHugeicon(Chatting01Icon),
  exchange: createHugeicon(ArrowDataTransferHorizontalIcon),
  question: createHugeicon(MessageQuestionIcon),
  arrowUp01: createHugeicon(ArrowUp01Icon),
  arrowDown01: createHugeicon(ArrowDown01Icon),
} satisfies Record<string, Icon>;

export const Icons = Object.fromEntries(
  Object.entries(_Icons).map(([key, Component]) => {
    (Component as React.ComponentType).displayName = key;
    return [key, Component];
  })
) as typeof _Icons;

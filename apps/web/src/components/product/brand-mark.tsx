import { cn } from "../../lib/utils";

export function BrandMark({ className }: Readonly<{ className?: string }>) {
  return (
    <span
      aria-hidden="true"
      className={cn(
        "grid size-8 shrink-0 place-items-center rounded-lg bg-primary text-sm font-bold tracking-tight text-primary-foreground shadow-xs",
        className,
      )}
    >
      A
    </span>
  );
}

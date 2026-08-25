import type { ReactNode } from "react";

import { cn } from "../../lib/utils";

export function PageFrame({
  children,
  className,
  width = "default",
}: Readonly<{
  children: ReactNode;
  className?: string;
  width?: "default" | "reading" | "full";
}>) {
  return (
    <div
      className={cn(
        "mx-auto w-full px-5 py-6 sm:px-6 sm:py-8 lg:px-8",
        width === "default" && "max-w-[90rem]",
        width === "reading" && "max-w-4xl",
        width === "full" && "max-w-none",
        className,
      )}
    >
      {children}
    </div>
  );
}

export function PageHeader({
  eyebrow,
  title,
  description,
  status,
  actions,
  className,
}: Readonly<{
  eyebrow?: ReactNode;
  title: ReactNode;
  description?: ReactNode;
  status?: ReactNode;
  actions?: ReactNode;
  className?: string;
}>) {
  return (
    <header
      className={cn(
        "mb-8 flex flex-col gap-5 border-b pb-6 md:flex-row md:items-start md:justify-between",
        className,
      )}
    >
      <div className="min-w-0 max-w-3xl">
        {eyebrow && (
          <div className="mb-1.5 text-xs font-medium text-muted-foreground">{eyebrow}</div>
        )}
        <div className="flex flex-wrap items-center gap-2.5">
          <h1>{title}</h1>
          {status}
        </div>
        {description && (
          <p className="mt-2 max-w-2xl text-sm leading-6 text-muted-foreground">{description}</p>
        )}
      </div>
      {actions && <div className="flex shrink-0 flex-wrap items-center gap-2">{actions}</div>}
    </header>
  );
}

export function SectionHeader({
  title,
  description,
  action,
}: Readonly<{ title: ReactNode; description?: ReactNode; action?: ReactNode }>) {
  return (
    <div className="mb-4 flex items-start justify-between gap-4">
      <div>
        <h2>{title}</h2>
        {description && <p className="mt-1 text-sm text-muted-foreground">{description}</p>}
      </div>
      {action}
    </div>
  );
}

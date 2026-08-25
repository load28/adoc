import { Link } from "@tanstack/react-router";
import { AlertCircle, Info, LoaderCircle } from "lucide-react";
import {
  createElement,
  type CSSProperties,
  type ElementType,
  type HTMLAttributes,
  type InputHTMLAttributes,
  type ReactNode,
} from "react";

import { cn } from "../../lib/utils";
import { Button as ShadcnButton, buttonVariants } from "../ui/button";
import { Input } from "../ui/input";
import { Textarea } from "../ui/textarea";

type Appearance =
  | "primary"
  | "subtle"
  | "danger"
  | "default"
  | "inprogress"
  | "success"
  | "removed"
  | "moved"
  | "warning"
  | "error"
  | "info";

export function Button({
  appearance,
  isLoading,
  isDisabled,
  spacing,
  isSelected,
  className,
  ...props
}: React.ComponentProps<typeof ShadcnButton> & {
  appearance?: Appearance;
  isLoading?: boolean;
  isDisabled?: boolean;
  spacing?: "compact" | "default";
  isSelected?: boolean;
}) {
  return (
    <ShadcnButton
      variant={
        appearance === "danger"
          ? "destructive"
          : appearance === "subtle"
            ? "ghost"
            : appearance === "primary"
              ? "default"
              : "outline"
      }
      size={spacing === "compact" ? "sm" : props.size}
      pending={isLoading}
      disabled={isDisabled}
      className={cn(isSelected && "bg-accent text-accent-foreground", className)}
      {...props}
    />
  );
}

export function LinkButton({
  href,
  appearance,
  isSelected,
  shouldFitContainer,
  spacing,
  className,
  children,
  ...props
}: Omit<React.ComponentProps<"a">, "href"> & {
  href: string;
  appearance?: Appearance;
  isSelected?: boolean;
  shouldFitContainer?: boolean;
  spacing?: "compact" | "default";
}) {
  const classes = cn(
    buttonVariants({
      variant: appearance === "primary" ? "default" : appearance === "subtle" ? "ghost" : "outline",
    }),
    isSelected && "bg-accent text-accent-foreground",
    shouldFitContainer && "w-full justify-start",
    spacing === "compact" && "h-8 px-3 text-xs",
    className,
  );
  if (href.startsWith("/") && !href.startsWith("//") && !href.startsWith("/api/")) {
    const url = new URL(href, "https://adoc.invalid");
    return (
      <Link
        to={url.pathname}
        search={Object.fromEntries(url.searchParams)}
        hash={url.hash.slice(1)}
        className={classes}
        {...props}
      >
        {children}
      </Link>
    );
  }
  return (
    <a href={href} className={classes} {...props}>
      {children}
    </a>
  );
}

const spaces: Record<string, string> = {
  "space.025": "gap-1",
  "space.050": "gap-1.5",
  "space.075": "gap-2",
  "space.100": "gap-2.5",
  "space.150": "gap-4",
  "space.200": "gap-6",
  "space.250": "gap-8",
  "space.300": "gap-10",
};

export function Stack({
  space = "space.100",
  alignInline,
  className,
  ...props
}: HTMLAttributes<HTMLDivElement> & { space?: string; alignInline?: "center" | "start" | "end" }) {
  return (
    <div
      className={cn(
        "flex min-w-0 flex-col",
        spaces[space] ?? "gap-2.5",
        alignInline === "center" && "items-center",
        alignInline === "end" && "items-end",
        className,
      )}
      {...props}
    />
  );
}

export function Inline({
  space = "space.100",
  shouldWrap,
  alignBlock,
  spread,
  className,
  ...props
}: HTMLAttributes<HTMLDivElement> & {
  space?: string;
  shouldWrap?: boolean;
  alignBlock?: "center" | "start" | "end";
  spread?: "space-between";
}) {
  return (
    <div
      className={cn(
        "flex min-w-0",
        spaces[space] ?? "gap-2.5",
        shouldWrap && "flex-wrap",
        alignBlock === "center" && "items-center",
        alignBlock === "end" && "items-end",
        spread === "space-between" && "justify-between",
        className,
      )}
      {...props}
    />
  );
}

export function Box({
  as = "div",
  padding,
  paddingBlockStart,
  className,
  ...props
}: HTMLAttributes<HTMLElement> & {
  as?: ElementType;
  padding?: string;
  paddingBlockStart?: string;
}) {
  const paddingClass =
    padding === "space.600"
      ? "p-12"
      : padding === "space.400"
        ? "p-8"
        : padding === "space.200"
          ? "p-6"
          : padding === "space.150"
            ? "p-4"
            : padding === "space.100"
              ? "p-2.5"
              : undefined;
  return createElement(as, {
    ...props,
    className: cn(paddingClass, paddingBlockStart && "pt-6", className),
  });
}

export function Text({
  size,
  weight,
  className,
  children,
  ...props
}: HTMLAttributes<HTMLSpanElement> & {
  size?: "small" | "medium";
  weight?: "bold" | "semibold" | "regular";
}) {
  return (
    <span
      className={cn(
        size === "small" ? "text-[13px] text-muted-foreground" : "text-sm",
        (weight === "bold" || weight === "semibold") && "font-semibold text-foreground",
        className,
      )}
      {...props}
    >
      {children}
    </span>
  );
}

export const Textfield = Input;
export const TextArea = Textarea;

export function Checkbox({
  label,
  isChecked,
  isDisabled,
  ...props
}: Omit<InputHTMLAttributes<HTMLInputElement>, "checked"> & {
  label?: ReactNode;
  isChecked?: boolean;
  isDisabled?: boolean;
}) {
  return (
    <label className="inline-flex min-h-8 items-center gap-2 text-sm">
      <input
        type="checkbox"
        checked={isChecked}
        disabled={isDisabled}
        className="size-4 rounded border-input accent-primary outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
        {...props}
      />
      {label && <span>{label}</span>}
    </label>
  );
}

export function Lozenge({
  appearance,
  children,
}: Readonly<{ appearance?: Appearance; children: ReactNode }>) {
  return (
    <span
      className={cn(
        "inline-flex h-6 items-center rounded-full border px-2 text-xs font-medium",
        appearance === "success" && "border-success/25 bg-success/10 text-success-foreground",
        appearance === "inprogress" && "border-info/25 bg-info/10 text-info-foreground",
        (appearance === "removed" || appearance === "error") &&
          "border-destructive/25 bg-destructive/10 text-destructive",
        appearance === "warning" && "border-warning/30 bg-warning/10 text-warning-foreground",
      )}
    >
      {children}
    </span>
  );
}

export function InlineMessage({
  appearance = "default",
  title,
  children,
}: Readonly<{
  appearance?: Appearance | "error" | "info";
  title: ReactNode;
  children?: ReactNode;
}>) {
  const error = appearance === "error" || appearance === "danger";
  const Icon = error ? AlertCircle : Info;
  return (
    <div
      role={error ? "alert" : "status"}
      className={cn(
        "grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 rounded-lg border p-3 text-sm",
        error ? "border-destructive/25 bg-destructive/8" : "border-info/25 bg-info/8",
      )}
    >
      <Icon
        aria-hidden="true"
        className={cn("mt-0.5 size-4", error ? "text-destructive" : "text-info")}
      />
      <strong className="font-medium">{title}</strong>
      {children && <div className="col-start-2 text-muted-foreground">{children}</div>}
    </div>
  );
}

export function EmptyState({
  header,
  description,
  primaryAction,
}: Readonly<{
  header: ReactNode;
  headingLevel?: number;
  description?: ReactNode;
  primaryAction?: ReactNode;
}>) {
  return (
    <div className="flex min-h-56 flex-col items-center justify-center rounded-lg border border-dashed bg-muted/25 px-6 text-center">
      <h2 className="text-lg font-semibold">{header}</h2>
      {description && <p className="mt-2 max-w-md text-sm text-muted-foreground">{description}</p>}
      {primaryAction && <div className="mt-5">{primaryAction}</div>}
    </div>
  );
}

export function Skeleton({
  width,
  height,
}: Readonly<{ width?: string | number; height?: string | number }>) {
  const style: CSSProperties = { width, height };
  return <div aria-hidden="true" className="animate-pulse rounded-md bg-muted" style={style} />;
}

export function Spinner() {
  return (
    <LoaderCircle aria-label="불러오는 중" className="size-5 animate-spin text-muted-foreground" />
  );
}

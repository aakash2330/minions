import type { ComponentProps, ReactNode } from "react";

import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { cn } from "@/lib/utils";

type PanelProps = ComponentProps<typeof PanelBody> & {
  title?: ReactNode;
  description?: ReactNode;
};

function PanelDefaultContent({
  title,
  description,
  children,
  className,
  ...props
}: PanelProps) {
  return (
    <>
      {(title || description) && (
        <PanelHeader>
          {title && <PanelTitle>{title}</PanelTitle>}
          {description && (
            <PanelDescription>{description}</PanelDescription>
          )}
        </PanelHeader>
      )}
      <PanelBody className={className} {...props}>
        {children}
      </PanelBody>
    </>
  );
}

type PanelRootProps = ComponentProps<typeof Sheet>;

function PanelRoot(props: PanelRootProps) {
  return <Sheet {...props} />;
}

type PanelContentProps = ComponentProps<typeof SheetContent>;

function PanelContent({
  className,
  ...props
}: PanelContentProps) {
  return (
    <SheetContent
      className={cn(
        "border-border/80 bg-popover p-0 text-popover-foreground",
        className,
      )}
      {...props}
    />
  );
}

type PanelHeaderProps = ComponentProps<typeof SheetHeader>;

function PanelHeader({
  className,
  ...props
}: PanelHeaderProps) {
  return (
    <SheetHeader
      className={cn("border-b border-border px-4 py-3", className)}
      {...props}
    />
  );
}

type PanelTitleProps = ComponentProps<typeof SheetTitle>;

function PanelTitle(props: PanelTitleProps) {
  return <SheetTitle {...props} />;
}

type PanelDescriptionProps = ComponentProps<typeof SheetDescription>;

function PanelDescription(props: PanelDescriptionProps) {
  return <SheetDescription {...props} />;
}

type PanelBodyProps = ComponentProps<"div">;

function PanelBody({
  className,
  ...props
}: PanelBodyProps) {
  return (
    <div
      className={cn(
        "flex min-h-0 flex-1 flex-col overflow-y-auto px-4 py-3",
        className,
      )}
      {...props}
    />
  );
}

export const Panel = Object.assign(PanelDefaultContent, {
  Root: PanelRoot,
  Content: PanelContent,
  Header: PanelHeader,
  Title: PanelTitle,
  Description: PanelDescription,
  Body: PanelBody,
});

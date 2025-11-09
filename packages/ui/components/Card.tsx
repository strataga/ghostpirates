import * as React from "react";

export interface CardProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode;
}

export function Card({ children, className = "", ...props }: CardProps) {
  return (
    <div
      className={`rounded-lg border border-gray-200 bg-white p-6 shadow-sm ${className}`}
      {...props}
    >
      {children}
    </div>
  );
}

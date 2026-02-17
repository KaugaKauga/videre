import { SVGProps } from "react";

export function ListTree(props: SVGProps<SVGSVGElement>) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      {...props}
    >
      <path stroke="none" d="M0 0h24v24H0z" fill="none" />
      <path d="M9 6h11" />
      <path d="M12 12h8" />
      <path d="M12 18h8" />
      <path d="M4 6a1 1 0 1 0 2 0a1 1 0 0 0 -2 0" />
      <path d="M7 12a1 1 0 1 0 2 0a1 1 0 0 0 -2 0" />
      <path d="M7 18a1 1 0 1 0 2 0a1 1 0 0 0 -2 0" />
    </svg>
  );
}

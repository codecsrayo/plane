declare module "next/script" {
  // Reuse React's standard <script> attribute typing instead of `[key: string]: any`.
  // Next.js overrides onLoad/onError to plain `() => void` (no React event arg), so we
  // omit those from the base and redeclare them with the looser signature.
  type ScriptProps = Omit<React.ScriptHTMLAttributes<HTMLScriptElement>, "onLoad" | "onError" | "children"> & {
    strategy?: "beforeInteractive" | "afterInteractive" | "lazyOnload" | "worker";
    onLoad?: () => void;
    onError?: () => void;
    children?: string;
  };

  const Script: React.FC<ScriptProps>;
  export default Script;
}

import { cn } from "@/lib/utils";

type ThemePreviewSwatchProps = {
  id: string;
  color: string;
  className?: string;
};

const DARK_THEME_IDS = new Set(["dark", "dark-one"]);
const DARK_THEME_SURFACES: Record<string, { shell: string; panel: string }> = {
  dark: { shell: "#09090b", panel: "#18181b" },
  "dark-one": { shell: "#1f232b", panel: "#2b303a" },
};

export function ThemePreviewSwatch({
  id,
  color,
  className,
}: ThemePreviewSwatchProps) {
  const isDarkPreview = DARK_THEME_IDS.has(id);
  const surface = DARK_THEME_SURFACES[id] ?? {
    shell: "#ffffff",
    panel: "#f8fafc",
  };
  const quietLine = isDarkPreview
    ? "rgba(255, 255, 255, 0.18)"
    : "rgba(15, 23, 42, 0.14)";
  const strongLine = isDarkPreview
    ? "rgba(255, 255, 255, 0.34)"
    : "rgba(15, 23, 42, 0.22)";

  return (
    <span
      aria-hidden="true"
      className={cn(
        "relative block h-10 w-14 shrink-0 overflow-hidden rounded-lg border shadow-sm",
        isDarkPreview ? "border-white/15" : "border-border/70",
        className,
      )}
      style={{
        background: `linear-gradient(135deg, ${surface.shell}, ${surface.panel})`,
      }}
    >
      <span
        className="absolute inset-x-0 top-0 h-1"
        style={{ backgroundColor: color }}
      />
      <span
        className="absolute bottom-1.5 left-1.5 top-2 w-2 rounded-sm"
        style={{ backgroundColor: quietLine }}
      />
      <span
        className="absolute left-5 right-1.5 top-2.5 h-1 rounded-full"
        style={{ backgroundColor: strongLine }}
      />
      <span
        className="absolute left-5 right-3 top-5 h-1 rounded-full"
        style={{ backgroundColor: color, opacity: 0.82 }}
      />
      <span
        className="absolute bottom-2 left-5 right-2 h-1 rounded-full"
        style={{ backgroundColor: quietLine }}
      />
    </span>
  );
}

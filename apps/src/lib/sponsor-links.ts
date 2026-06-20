// 作者页远程内容（赞助/服务器推荐）的类型与归一化逻辑。
// 注意：本模块不包含任何硬编码广告或推广数据，仅负责把后端运行时返回的
// 作者内容数组归一化为稳定结构，供作者页渲染。广告/推广入口已在清理阶段移除。

export interface SponsorLinkItem {
  key: string;
  name: string;
  description: string;
  href: string;
  actionLabel: string;
  imageSrc?: string;
  imageAlt?: string;
}

/** 判断值是否为普通对象记录。 */
function asRecord(value: unknown): Record<string, unknown> | null {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : null;
}

/** 转为去除首尾空白的字符串，非字符串返回空串。 */
function asTrimmedString(value: unknown): string {
  return typeof value === "string" ? value.trim() : "";
}

/** 归一化可选文本：空串归一化为 undefined。 */
function normalizeOptionalText(value: unknown): string | undefined {
  const normalized = asTrimmedString(value);
  return normalized || undefined;
}

/** 浅拷贝作者内容数组，避免外部引用被意外修改。 */
export function cloneSponsorLinkItems(
  items: readonly SponsorLinkItem[],
): SponsorLinkItem[] {
  return items.map((item) => ({ ...item }));
}

/**
 * 将后端返回的任意值归一化为 SponsorLinkItem 数组。
 *
 * 参数:
 *   value: 后端运行时返回的原始值（期望为数组）。
 *   fallback: 当 value 非数组时返回的回退数组（默认空数组）。
 * 返回:
 *   归一化后的 SponsorLinkItem 数组。
 */
export function normalizeSponsorLinkItems(
  value: unknown,
  fallback: readonly SponsorLinkItem[] = [],
): SponsorLinkItem[] {
  if (!Array.isArray(value)) {
    return cloneSponsorLinkItems(fallback);
  }

  return value.map((item, index) => {
    const source = asRecord(item) ?? {};
    return {
      key: asTrimmedString(source.key) || `item-${index + 1}`,
      name: asTrimmedString(source.name),
      description: asTrimmedString(source.description),
      href: asTrimmedString(source.href),
      actionLabel: asTrimmedString(source.actionLabel),
      imageSrc: normalizeOptionalText(source.imageSrc),
      imageAlt: normalizeOptionalText(source.imageAlt),
    } satisfies SponsorLinkItem;
  });
}

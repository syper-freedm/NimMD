// ruleid: nimmd-katex-trust
const a = { trust: true };
// ruleid: nimmd-katex-trust
const b = { trust: () => true };
// ruleid: nimmd-katex-trust
const c = { trust : (ctx: unknown) => ctx !== null };
// ok: nimmd-katex-trust
const d = { trust: false };
// ok: nimmd-katex-trust
const e = { strict: true, trusted: 1 };

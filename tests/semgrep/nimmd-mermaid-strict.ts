declare const mermaid: { initialize(cfg: object): void };
declare const cfg: object;
// ruleid: nimmd-mermaid-strict
mermaid.initialize({ startOnLoad: false });
// ruleid: nimmd-mermaid-strict
mermaid.initialize( { securityLevel: "loose" });
// ruleid: nimmd-mermaid-strict
mermaid.initialize(cfg);
// ok: nimmd-mermaid-strict
mermaid.initialize({ securityLevel: 'strict' });
// ok: nimmd-mermaid-strict
mermaid.initialize( { themeVariables: { a: 1 }, securityLevel: "strict", startOnLoad: false });

// Заглушка вместо elkjs: его лицензия EPL-2.0 вне белого списка (спек 12), в бандл он не попадает.
// Контракт для хостов (#43, #53): бандл без ELK, хост переписывает `layout: elk*` → dagre до вызова mermaid.
export default class ELK {
  layout() {
    return Promise.reject(new Error("ELK layout is not available in NimMD"));
  }
}

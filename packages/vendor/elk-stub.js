// Заглушка вместо elkjs: его лицензия EPL-2.0 вне белого списка (спек 12), в бандл он не попадает.
// Сборка Mermaid выбирает dagre для настроек `layout: elk*`. Заглушка исключает elkjs из бандла.
export default class ELK {
  layout() {
    return Promise.reject(new Error("ELK layout is not available in NimMD"));
  }
}

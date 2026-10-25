declare const el: HTMLElement;
declare const html: string;
// ruleid: nimmd-dom-html-sink
el.innerHTML = html;
// ruleid: nimmd-dom-html-sink
el.outerHTML = html;
// ruleid: nimmd-dom-html-sink
el.insertAdjacentHTML("beforeend", html);
// ruleid: nimmd-dom-html-sink
document.write(html);
// ok: nimmd-dom-html-sink
el.innerHTML = "";
// ok: nimmd-dom-html-sink
el.textContent = html;


customElements.define('nav-bar', class NavBar extends HTMLElement {
  constructor() {
    super()
  }
  connectedCallback() {
    const root = this.attachShadow({ mode: 'closed' })
    root.innerHTML = `
        <ul hx-get="/navbar" hx-trigger="load" hx-swap="innerHTML"></ul>
    `
    // htmx.process(root)

  }

})

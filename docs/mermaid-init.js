// Renders the ```mermaid fences that the mdbook-mermaid preprocessor turns
// into <pre class="mermaid"> blocks. Mermaid itself is loaded from jsDelivr
// rather than vendored: `mdbook-mermaid install` would drop a 2.7 MB
// mermaid.min.js into this directory, and the site already reaches the same
// CDN for the KaTeX stylesheet. The theme follows the book's: mdBook stamps
// its theme name as a class on <html>, and a change re-renders on reload.
(async () => {
    const darkThemes = ['ayu', 'navy', 'coal'];
    const classList = document.documentElement.classList;
    const dark = [...classList].some((c) => darkThemes.includes(c));
    const { default: mermaid } = await import(
        'https://cdn.jsdelivr.net/npm/mermaid@11.4.1/dist/mermaid.esm.min.mjs'
    );
    mermaid.initialize({ startOnLoad: false, theme: dark ? 'dark' : 'default' });
    await mermaid.run({ querySelector: 'pre.mermaid' });
})();

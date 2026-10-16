// Installed into the Rust core's page (rust-viewer/, the prototype viewer on
// the Rust core) after it loads: the same `window.__bench` protocol as
// js.html, over the page's `window.__rust` hook. Actions go through the
// viewer's store, as its renderers send them, under the Rust core's action
// names (a point's `move`, a text input's `updateImmediateValue`); it has
// no math input, so typing into one is a gap.
(() => {
    const nextPaint = () =>
        new Promise((ok) => requestAnimationFrame(() => setTimeout(ok, 0)));
    const nextFrame = () =>
        new Promise((ok) => requestAnimationFrame(() => ok(performance.now())));
    const board = () => Object.values(window.JXG?.boards ?? {})[0] ?? null;
    window.__bench = {
        ready: false,
        core: "rust",
        /** Text to first render: the core's first flat DAST, then `waitFor` on the page. */
        async load(text, waitFor = {}) {
            while (!window.__rust)
                await new Promise((ok) => setTimeout(ok, 20));
            const t0 = performance.now();
            await window.__rust.loadSource(text);
            const present = () =>
                waitFor.board
                    ? (board()?.objectsList?.length ?? 0) > 0
                    : waitFor.id
                      ? document.getElementById(waitFor.id) !== null
                      : document.querySelector("#root [id]") !== null;
            while (!present()) await nextPaint();
            await nextPaint();
            this.ready = true;
            return performance.now() - t0;
        },
        /**
         * Client coordinates of a named point: its coordinates from the
         * viewer's flat DAST, and the nearest JSXGraph point at them (the
         * prototype gives every point the same label) kept for `position`.
         */
        async locate(name) {
            const b = board();
            if (!b) return null;
            const idx = await window.__rust.resolve(name);
            if (idx === -1) return null;
            const xy = window.__rust.pointCoords(idx);
            if (!xy) return null;
            const c = new window.JXG.Coords(window.JXG.COORDS_BY_USER, xy, b)
                .scrCoords;
            let best = null,
                bestD = Infinity;
            for (const o of b.objectsList) {
                if (o.elType !== "point" || !o.visProp.visible) continue;
                const d = Math.hypot(
                    o.coords.scrCoords[1] - c[1],
                    o.coords.scrCoords[2] - c[2],
                );
                if (d < bestD) {
                    best = o;
                    bestD = d;
                }
            }
            this._pt = { board: b, el: best };
            return this.position();
        },
        position() {
            const p = this._pt;
            if (!p?.el) return null;
            p.board.containerObj.scrollIntoView({ block: "center" });
            const r = p.board.containerObj.getBoundingClientRect();
            return [
                r.left + p.el.coords.scrCoords[1],
                r.top + p.el.coords.scrCoords[2],
            ];
        },
        async act(step) {
            const componentIdx = await window.__rust.resolve(step.target);
            if (componentIdx === -1)
                throw new Error(`target ${step.target} did not resolve`);
            const component = window.__rust.componentType(componentIdx);
            const call = (actionName, args) =>
                window.__rust.dispatch({
                    component,
                    actionName,
                    componentIdx,
                    ...(args ? { args } : {}),
                });
            switch (step.action) {
                case "movePoint":
                    return call("move", { x: step.x, y: step.y });
                case "updateValue":
                    if (component !== "textInput")
                        throw new Error(
                            `unsupported: typing into a ${component} (the Rust core has no mathInput)`,
                        );
                    await call("updateImmediateValue", { text: step.value });
                    return call("updateValue");
                default:
                    throw new Error(
                        `unsupported action ${step.action} in the Rust core`,
                    );
            }
        },
        nextFrame,
        nextPaint,
    };
})();

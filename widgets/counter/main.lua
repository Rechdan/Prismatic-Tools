-- Counter — the first Prismatic Tools widget. Proves the host<->widget render/
-- state loop: the click count lives in `state`, the button mutates it, and the
-- runtime re-renders `render(state)`.
--
-- Contract (widget-runtime): return a module table with a plain-data `state`
-- table (held by the host for future persistence) and a `render(state)` function
-- that returns a UI tree built from the injected `vstack`/`hstack`/`text`/`button`
-- builders (hash keys = props, array entries = children).
return {
  state = { count = 0 },
  render = function(state)
    return vstack {
      spacing = 12,
      text('Demo tool — proves the host render/state loop.'),
      text('clicks: ' .. state.count),
      button('Click me', function()
        state.count = state.count + 1
      end),
    }
  end,
}

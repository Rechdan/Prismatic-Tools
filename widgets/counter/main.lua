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
  -- Display-only nav preview: a custom card that mirrors the click count. Shares
  -- the same `state` as `render`, so clicking in the main view updates it live.
  -- Returning a `border` root styles the tapped nav card (padding + rounding on
  -- top of the theme-aware default fill).
  nav = function(state)
    return border {
      padding = 10,
      corner_radius = 6,
      vstack {
        spacing = 4,
        text('Counter'),
        text('clicks: ' .. state.count),
      },
    }
  end,
}

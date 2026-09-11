# Everarc design direction

Everarc uses a calm sketchbook-dashboard visual language for generated HTML.
It favors warm paper, dark graphite ink, and modest highlighter accents over a
traditional dense-finance interface.

## Foundations

- **Canvas:** warm cream paper with a subtle dotted drafting grid.
- **Ink:** near-black graphite for text, outlines, and structural lines; muted
  graphite for secondary labels.
- **Accents:** lavender for progress, peach for a gentle emphasis wash, mint
  for positive changes, and sky blue for supporting marks.
- **Typography:** Bricolage Grotesque for display headings, Plus Jakarta Sans
  for body copy, and Space Mono for labels and financial figures.
- **Depth:** mostly borderless paper cards with soft, low-contrast neutral
  shadows. Use strong ink lines inside charts and small details, not as heavy
  frames around every surface. Do not use gradients.
- **Imperfection:** restrained rotations and slightly asymmetric radii give
  cards a hand-arranged quality without harming readability.

## Initial components

The first generated dashboard contains three reusable template components:

1. Total net worth canvas
2. Milestone progress card
3. Chart canvas

Their values are mocked until the TOML schema provides real financial data.
Do not add navigation, editing controls, headers, footers, or detailed
financial views unless a later feature explicitly requires them.

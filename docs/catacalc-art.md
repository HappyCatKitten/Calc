# Catacalc artwork

Generated with the built-in imagegen tool for the optional Catacalc theme.
Asset: `qml/assets/catacalc-banner.png`.

Prompt: Create a standalone illustration asset for the Catacalc calculator theme based on the cute caterpillar in the reference. Only the artwork, NO interface, NO numbers, NO text, NO buttons, NO border. Very wide panoramic 3:1 composition. Adorable lime green segmented caterpillar with big glossy eyes, smiling face, rosy cheeks, delicate antennae and tiny yellow feet resting on a lush emerald leaf with beautiful dew drops. Caterpillar centered horizontally with entire head antennae and body comfortably inside frame. Rich botanical leaves framing edges, soft sunlit green woodland bokeh behind. Premium polished tactile 3D storybook illustration, match reference character and gorgeous green colors. This will be a shallow header banner so keep character comfortably centered and relatively small with plentiful background space.


## Matching skin and circular controls

`qml/assets/catacalc-skin.png` is the approved mockup with labels and icons removed, generated using the built-in imagegen tool. QML renders the mascot, panels and botanical illustrations from this texture atlas. All calculator content and interactions are live. The original rectangular key surfaces are covered and replaced by native circular controls, following the subsequent request for all buttons to be circles.

Skin prompt: Use case: precise-object-edit. This exact image is the edit target. Create a clean UI skin texture for implementation. Keep EXACT same canvas dimensions, composition, every panel, button, key position and geometry, all shadows, lighting, colors, the adorable caterpillar EXACTLY unchanged, all leaf and flower artwork unchanged. REMOVE ALL TEXT AND SYMBOLS from the entire image by seamlessly filling with the existing underlying material: remove Catercalc letters, hamburger icon, tiny leaf logo, window minimize/maximize/close symbols, clock icon, expression, big number result, all letters/numbers/operators on every key, History heading, chevron, magnifying glass and search placeholder, all history entries and their numerals. Keep every button (including yellow buttons and tall green equals key) and every empty cream panel and search box in precisely original position. Keep the thin separators in history. The output should look like the identical original design with every text label and UI icon erased to leave blank control surfaces. Do not redesign, do not add anything, do not shift or scale anything. This is a precision blanking edit of the supplied image, not a new design.

Validation: `python3 scripts/catacalc-smoke.py` checks the native GUI under Xvfb: arithmetic with mouse hit targets, expression editing, filtered history recall, scientific mode, compact mode, restart persistence, theme-off regression, and empty Qt diagnostic logs. Set CATACALC_TEST_LAUNCHER to exercise an installed bundle.

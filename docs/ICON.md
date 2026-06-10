# App icon

## What's in the repo

- **Launcher icon (adaptive, API 26+):**
  - `app/src/main/res/mipmap-anydpi-v26/ic_launcher.xml` + `ic_launcher_round.xml`
    — adaptive icon = blue background layer + logo foreground layer.
  - `app/src/main/res/values/ic_launcher_background.xml` — background color
    `#08589D` (the app blue).
  - `app/src/main/res/mipmap-xxxhdpi/ic_launcher_foreground.png` — 432×432, the
    saxophone/note/magnifier logo on transparency, centered at ~67% so it stays
    inside the launcher's mask (circle/squircle) without clipping.
  - The manifest references `@mipmap/ic_launcher` / `@mipmap/ic_launcher_round`.
  - `minSdk = 28` (≥ 26), so the adaptive icon always applies — no legacy raster
    fallback is needed.

- **Play Store listing icon:** `docs/play-store-icon-512.png` — 512×512, 32-bit,
  full-bleed blue (Google applies the rounded-corner mask + shadow). Upload this
  in the Play Console; it is **not** packaged in the app.

## Why these were regenerated

The original `app/src/main/res/drawable/tunecall.png` (1024×1024) had a
**transparency checkerboard baked in as opaque pixels** around the blue rounded
square (it was exported over a checkerboard and flattened). That can't be used
directly as a clean icon. The assets above were derived from it by flood-filling
the checkerboard to the app blue, then extracting the logo onto transparency.
`tunecall.png` is left in place but is no longer referenced.

## Regenerating (ImageMagick)

```bash
SRC=app/src/main/res/drawable/tunecall.png
# 1. Replace the baked-in outer checkerboard with the app blue (full bleed):
magick "$SRC" -fuzz 30% -fill "#08589D" -draw "color 0,0 floodfill" bleed.png
# 2. Play Store icon (full-bleed, no alpha):
magick bleed.png -resize 512x512 -alpha remove -alpha off docs/play-store-icon-512.png
# 3. Extract the logo onto transparency:
magick bleed.png -fuzz 22% -fill none -draw "alpha 0,0 floodfill" -trim +repage logo.png
# 4. Adaptive foreground (logo centered at ~67% of a 432 canvas):
magick logo.png -resize 290x290 -background none -gravity center -extent 432x432 \
  app/src/main/res/mipmap-xxxhdpi/ic_launcher_foreground.png
```

## Optional polish (not done)

- **Themed/monochrome icon** (Android 13+): add a `<monochrome>` layer to the
  adaptive icon (a single-color silhouette of the logo) so it tints with the
  system theme. Optional.
- **Cleaner source art**: if you have the original vector (sax + note +
  magnifier), exporting foreground/background from that beats deriving from the
  flattened PNG.
- **Lower-density foreground buckets**: only `xxxhdpi` (432px) is provided;
  Android downscales it for lower densities. Adding `xxhdpi`/`xhdpi` variants
  trims runtime memory slightly but isn't required.

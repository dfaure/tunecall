# Play Store screenshots (phone)

Google Play requires **2–8 phone screenshots**. These are real captures of the
app running on the Android device — there is no automated step.

## Spec (what Play accepts)

- PNG or JPEG, **24-bit, no alpha** (a normal Android screenshot is fine).
- Each side **320–3840 px**.
- Long side **no more than 2×** the short side (a normal phone portrait shot,
  e.g. 1080×1920 or 1080×2400, is well within this).
- Capture the **phone** section on the phone (native portrait — no resizing
  needed). Reading charts on a phone is cramped, but the screenshots only need
  to show the UI working, so it's fine for the listing.
- The tablet is the better device for the separate "tablette" screenshot
  section (landscape / larger charts). Tablet shots also pass the phone
  validator, but portrait phone shots look best in the phone section.

## Shots to take (in this order)

Take them with content already loaded (Reload first, install at least one PDF),
so the app doesn't look empty. Capture with **Power + Volume-Down**.

1. **Search with results** — Search tab, type a common word (e.g. `blue` or
   `night`) so the list shows several hits with their "Book · p.N" subtitles.
   This is the headline shot: it shows the core feature at a glance.
2. **Page viewer on a chart** — open a result so a real scanned page fills the
   screen. Shows that it opens straight to the right page. Pick a clean,
   readable chart (not a blank/intro page).
3. **Books tab** — shows the list of supported books and which PDFs are
   installed. Signals it's a real library tool, and sets the "bring your own
   PDFs" expectation.
4. **Setlist** — a setlist open in the editor (a few songs listed) or playing.
   Shows the gig/rehearsal workflow.

2 is the minimum (1 + 2 cover the core). 4 makes a fuller listing.

## After capturing

Drop the raw PNGs into `docs/raw-screens/` and tell Claude. They will be
post-processed to a Play-ready set: alpha stripped, dimensions verified, and
padded to a valid aspect ratio only if needed. The reference command is:

```bash
# per file: flatten any alpha onto white, keep 8-bit RGB
magick raw.png -background white -alpha remove -alpha off -depth 8 ready.png
```

## Tablet screenshots (7" and 10")

Once tablet is a supported form factor, the **7" slot becomes required** (the
field is marked with a `*`), and **both** tablet slots demand a **stricter
aspect ratio than phones**: exactly **16:9 or 9:16** (a phone shot's ~19.5:9
will be rejected). The size brackets are:

- **7"**: each side **320-3840 px**.
- **10"**: each side **1080-7680 px**.

Because those brackets overlap, **one file can fill both slots**: keep the long
side <= 3840 (the 7" ceiling) and the short side >= 1080 (the 10" floor), i.e.
roughly 1080-2160 x 1920-3840 for portrait. So upload the *same* normalized
image to both the 7" and the 10" section.

Real tablet screens are almost never exactly 16:9/9:16 (usually 16:10), so the
raw capture has to be padded. Capture on the tablet, then run:

```bash
docs/adapt-tablet-screenshot.sh docs/screenshot-tablet-*.jpg
```

It keeps orientation, **pads (never crops)** to an exact 16:9/9:16 canvas sized
to satisfy both slots, downscales only if the input is bigger than that box,
strips alpha, and writes `<name>-916.jpg` (portrait) / `<name>-169.jpg`
(landscape) next to each input. The pad color defaults to the image's top-left
pixel so the letterbox blends with the app's light background (override with
`PAD_COLOR`; JPEG quality with `QUALITY`). Upload each output to **both** tablet
slots.

## Public-domain chart for the page-viewer shot

The scanned charts are copyrighted even when the *song* is public domain, so the
page-viewer screenshot is the one copyright-sensitive shot. Use **Silent Night**
(composed 1818) from **the Public Domain Christmas Jazz Fakebook** (`tpdxmasjfb`)
-- that whole book is public-domain by design, so even its arrangement is safe.

## Tips

- Use the phone in its normal portrait orientation.
- Hide anything personal (the page viewer shows the chart, which is fine).
- Consistent content across shots (same book/era of tunes) looks more polished.
- Don't add fake phone frames; Play shows them plainly.

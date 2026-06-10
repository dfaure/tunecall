# Adding your PDF books

TuneCall doesn't come with any sheet music — it searches **your own** scanned
fake-book PDFs and opens them at the right page. You add those PDFs once, by
copying them from a computer to your device over USB. After that, everything
(search, setlists, page turning) works offline.

You only need to do this when you add a new book.

## What you need

- Your fake-book **PDF files**.
- A **USB cable** to connect your phone/tablet to a computer.
- A few minutes.

## Step 1 — See which books are supported, and their exact names

Open TuneCall and go to the **Books** tab. It lists every book the app knows
how to index. Each entry shows a name like `realbk1h` — that is the **file name**
your PDF must use.

> The file name matters. A book stays greyed out as **"PDF missing"** until a
> PDF with the matching name is present. So `The Real Book Vol 1` must be saved
> as **`realbk1h.pdf`**, not `Real Book 1.pdf`.

(Capitalization doesn't matter — `RealBk1h.PDF` works too — but the rest of the
name must match exactly.)

## Step 2 — Connect your device to the computer

1. Plug the device into the computer with the USB cable.
2. On the device, a USB notification appears — tap it and choose
   **File transfer** (also called **MTP**). If you skip this, the computer only
   charges the device and won't see its files.

## Step 3 — Open the TuneCall folder on the device

On the computer, open the device's storage, then navigate to:

```
Internal storage / Android / data / fr.davidfaure.tunecall / files / pdfs
```

- **Windows**: open *This PC* → your device → *Internal storage* → the path
  above.
- **macOS**: install *Android File Transfer* (from android.com), open it, then
  the path above.
- **Linux**: open your file manager; the device appears as an MTP device.

If the `pdfs` folder isn't there yet, launch TuneCall on the device once (that
creates it) and reconnect.

## Step 4 — Copy your PDFs in

Drag your PDF files into that `pdfs` folder, making sure each file is named to
match a book from the Books tab (Step 1) — e.g. `realbk1h.pdf`.

You can copy several books at once.

## Step 5 — Load them in the app

Unplug the cable, open TuneCall, and tap **Reload** (on the Books tab). The books
you added turn from "PDF missing" to **installed**, and their songs become
searchable.

That's it — search for any song and tap a result to open the book at that page.

## Troubleshooting

- **The book still says "PDF missing" after Reload.** The file name doesn't match
  the book name in the Books tab. Rename the PDF (e.g. `realbk1h.pdf`) and tap
  Reload again.
- **I can't find the `Android/data` folder.** Make sure you chose *File
  transfer / MTP* on the device (Step 2), and that you launched TuneCall at least
  once so the folder exists.
- **Nothing happens on the computer when I plug in.** Try a different cable or
  port — some cables are charge-only. Then re-check the USB notification.

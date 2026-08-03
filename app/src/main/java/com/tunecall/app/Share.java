package com.tunecall.app;

import android.app.Activity;
import android.content.Intent;
import android.net.Uri;
import android.util.Log;

import androidx.core.content.FileProvider;

import java.io.File;

// One-shot "share this PDF" helper. Called from Rust via JNI
// (see src/share.rs) with a file the export step just wrote into the app's
// internal share/ dir. Wraps the file in a FileProvider content:// URI
// (file:// URIs are blocked from being shared on API 24+) and launches the
// system chooser. Runs on the UI thread — startActivity from a background
// thread crashes with a "must call from UI thread" IllegalStateException.
public final class Share {
    private static final String TAG = "TuneCallShare";
    private static final String AUTHORITY = "com.tunecall.app.fileprovider";

    private Share() {
    }

    public static void sharePdf(final Activity activity, final String path, final String subject) {
        if (activity == null || path == null) {
            return;
        }
        activity.runOnUiThread(new Runnable() {
            @Override
            public void run() {
                launch(activity, path, subject);
            }
        });
    }

    private static void launch(Activity activity, String path, String subject) {
        try {
            Uri uri = FileProvider.getUriForFile(activity, AUTHORITY, new File(path));
            Intent send = new Intent(Intent.ACTION_SEND)
                    .setType("application/pdf")
                    .putExtra(Intent.EXTRA_STREAM, uri)
                    // Every recipient of the chooser needs the read grant, so
                    // put it on the Intent itself, not on individual targets.
                    .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION);
            if (subject != null && !subject.isEmpty()) {
                // Some targets (mail, notes) surface this as the subject or
                // filename hint; others ignore it.
                send.putExtra(Intent.EXTRA_SUBJECT, subject);
            }
            Intent chooser = Intent.createChooser(send, "Share song");
            activity.startActivity(chooser);
        } catch (Exception e) {
            Log.w(TAG, "share failed for " + path, e);
        }
    }
}

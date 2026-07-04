package com.tunecall.app;

import android.app.Activity;
import android.content.Intent;
import android.database.Cursor;
import android.net.Uri;
import android.os.Bundle;
import android.provider.DocumentsContract;
import android.util.Log;

import java.io.File;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.io.OutputStream;

// One-shot SAF folder-import helper for the Books-tab "Import from folder…"
// button. NativeActivity can't easily receive onActivityResult, so this
// translucent Activity trampolines: onCreate launches ACTION_OPEN_DOCUMENT_TREE,
// onActivityResult copies every *.pdf in the chosen tree into
// getExternalFilesDir(null)/pdfs (the same path Rust exposes as pdf_dir), then
// tells the native side how many landed. Called from Rust via
// src/import.rs::launch_folder_picker; the notification comes back through the
// native symbol declared below.
public final class Import extends Activity {
    private static final String TAG = "TuneCallImport";
    private static final int PICK_REQUEST = 0x7c;

    // NativeActivity loads libtunecall.so at process start, so the symbol is
    // already resolvable by the time this class is used — but load it here
    // defensively in case class-init runs first (a JVM UnsatisfiedLinkError
    // would kill the copy thread and take the app down with it).
    static {
        try {
            System.loadLibrary("tunecall");
        } catch (Throwable t) {
            Log.w(TAG, "loadLibrary(tunecall) failed", t);
        }
    }

    // Fired from the copy thread when the import finishes. Count is the number
    // of PDFs imported, or -1 when the picker couldn't be launched at all.
    public static native void nativeOnImported(int count);

    // Called from Rust (JNI) to start the trampoline. Stacks on top of the
    // caller so finishing it returns to NativeActivity (NEW_TASK would put us
    // in a separate task and land the user back on the home screen).
    public static void launchPicker(final Activity host) {
        Intent i = new Intent(host, Import.class);
        host.startActivity(i);
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        Intent picker = new Intent(Intent.ACTION_OPEN_DOCUMENT_TREE);
        picker.addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION);
        try {
            startActivityForResult(picker, PICK_REQUEST);
        } catch (Exception e) {
            Log.w(TAG, "no SAF picker available", e);
            nativeOnImported(-1);
            finish();
        }
    }

    @Override
    protected void onActivityResult(int req, int res, Intent data) {
        super.onActivityResult(req, res, data);
        if (req != PICK_REQUEST) {
            finish();
            return;
        }
        // User cancelled / picker returned nothing: don't touch the UI at all
        // (no "0 imported" flash), just finish silently.
        if (res != RESULT_OK || data == null || data.getData() == null) {
            finish();
            return;
        }
        final Uri tree = data.getData();
        // Copy work happens off the UI thread so a big book doesn't ANR. The
        // Activity stays alive until the thread finishes.
        new Thread(new Runnable() {
            @Override
            public void run() {
                int count = -1;
                try {
                    count = copyPdfs(tree);
                } catch (Throwable t) {
                    Log.w(TAG, "SAF import crashed", t);
                }
                nativeOnImported(count);
                finish();
            }
        }, "TuneCallImport").start();
    }

    // Iterate the picked SAF tree and copy every top-level `.pdf` into pdf_dir.
    // Not recursive: books normally live at the top of the folder the user picked.
    private int copyPdfs(Uri tree) {
        File pdfDir = new File(getExternalFilesDir(null), "pdfs");
        if (!pdfDir.isDirectory() && !pdfDir.mkdirs()) {
            Log.w(TAG, "cannot create " + pdfDir);
            return -1;
        }
        Uri childrenUri = DocumentsContract.buildChildDocumentsUriUsingTree(
                tree, DocumentsContract.getTreeDocumentId(tree));
        String[] proj = new String[] {
                DocumentsContract.Document.COLUMN_DOCUMENT_ID,
                DocumentsContract.Document.COLUMN_DISPLAY_NAME,
        };
        int count = 0;
        Cursor c = null;
        try {
            c = getContentResolver().query(childrenUri, proj, null, null, null);
            if (c == null) {
                Log.w(TAG, "querying children returned null cursor");
                return -1;
            }
            while (c.moveToNext()) {
                String docId = c.getString(0);
                String name = c.getString(1);
                if (name == null || !name.toLowerCase().endsWith(".pdf")) {
                    continue;
                }
                Uri fileUri = DocumentsContract.buildDocumentUriUsingTree(tree, docId);
                File dst = new File(pdfDir, name);
                if (copyOne(fileUri, dst)) {
                    Log.i(TAG, "imported " + name);
                    count++;
                }
            }
        } finally {
            if (c != null) {
                c.close();
            }
        }
        return count;
    }

    private boolean copyOne(Uri src, File dst) {
        InputStream in = null;
        OutputStream out = null;
        try {
            in = getContentResolver().openInputStream(src);
            if (in == null) {
                return false;
            }
            out = new FileOutputStream(dst);
            byte[] buf = new byte[64 * 1024];
            int n;
            while ((n = in.read(buf)) > 0) {
                out.write(buf, 0, n);
            }
            return true;
        } catch (Exception e) {
            Log.w(TAG, "copy " + dst.getName() + " failed", e);
            return false;
        } finally {
            try { if (in != null) in.close(); } catch (Exception ignored) {}
            try { if (out != null) out.close(); } catch (Exception ignored) {}
        }
    }
}

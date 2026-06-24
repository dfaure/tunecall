package com.tunecall.app;

import android.app.Activity;
import android.os.Build;
import android.view.View;
import android.view.WindowInsets;
import android.view.WindowInsetsController;

// Toggles immersive fullscreen: hides the status and navigation bars while the
// PDF viewer is open (swiping from an edge brings them back transiently) and
// restores them otherwise. Called from Rust via JNI (see src/immersive.rs).
// Window/View access must run on the UI thread, so the change is posted there
// with runOnUiThread.
public final class Immersive {
    private Immersive() {
    }

    public static void setImmersive(final Activity activity, final boolean enabled) {
        if (activity == null) {
            return;
        }
        activity.runOnUiThread(new Runnable() {
            @Override
            public void run() {
                apply(activity, enabled);
            }
        });
    }

    private static void apply(Activity activity, boolean enabled) {
        View decor = activity.getWindow().getDecorView();
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            WindowInsetsController controller = decor.getWindowInsetsController();
            if (controller == null) {
                return;
            }
            if (enabled) {
                // Transient: a swipe from the edge reveals the bars briefly,
                // then they auto-hide again.
                controller.setSystemBarsBehavior(
                        WindowInsetsController.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE);
                controller.hide(WindowInsets.Type.systemBars());
            } else {
                controller.show(WindowInsets.Type.systemBars());
            }
        } else {
            // API 28-29: the pre-WindowInsetsController immersive flags.
            if (enabled) {
                decor.setSystemUiVisibility(
                        View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY
                                | View.SYSTEM_UI_FLAG_HIDE_NAVIGATION
                                | View.SYSTEM_UI_FLAG_FULLSCREEN
                                | View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION
                                | View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN);
            } else {
                decor.setSystemUiVisibility(
                        View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN
                                | View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION);
            }
        }
    }
}

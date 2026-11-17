package com.zevryn.app.screens.splash

import android.content.Intent
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.util.Log
import android.widget.ProgressBar
import androidx.appcompat.app.AppCompatActivity
import com.zevryn.app.MainActivity
import com.zevryn.app.R

/**
 * Brief launch screen for the Zevryn launcher.
 *
 * The reference app plays a locale GIF (`launcher_en.gif` / `launcher_zh.gif`)
 * here; the portable build stays offline and lightweight: a stylised "Zevryn"
 * title plus a short animated progress bar. After ~1.2 s it hands off to
 * [MainActivity] and finishes so the splash never stays in the back stack.
 *
 * NOTE: this activity is deliberately NOT registered as MAIN/LAUNCHER — the
 * parent session owns the manifest. Only the class is provided here.
 */
class SplashActivity : AppCompatActivity() {

    private val handler = Handler(Looper.getMainLooper())
    private var navigated = false

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.splash_activity)

        val progress = findViewById<ProgressBar>(R.id.splashProgress)
        progress.max = PROGRESS_MAX
        progress.progress = 0

        // Drive a short 0 -> 100 fill animation, then leave the splash.
        for (step in 1..PROGRESS_STEPS) {
            val value = PROGRESS_MAX * step / PROGRESS_STEPS
            handler.postDelayed({ safeSetProgress(progress, value) }, STEP_DELAY_MS * step)
        }
        handler.postDelayed({ navigateToLauncher() }, SPLASH_DURATION_MS)
    }

    /** Never let a bad progress update crash the launch flow. */
    private fun safeSetProgress(progress: ProgressBar, value: Int) {
        try {
            progress.progress = value
        } catch (t: Throwable) {
            Log.w(TAG, "Could not update splash progress", t)
        }
    }

    /** Navigate exactly once, and always finish the splash even on failure. */
    private fun navigateToLauncher() {
        if (navigated) return
        navigated = true
        try {
            startActivity(Intent(this, MainActivity::class.java))
        } catch (t: Throwable) {
            Log.e(TAG, "Could not start MainActivity", t)
        } finally {
            finish()
        }
    }

    override fun onDestroy() {
        handler.removeCallbacksAndMessages(null)
        super.onDestroy()
    }

    companion object {
        private const val TAG = "Zevryn"
        private const val SPLASH_DURATION_MS = 1200L
        private const val PROGRESS_STEPS = 12
        private const val PROGRESS_MAX = 100
        private const val STEP_DELAY_MS = SPLASH_DURATION_MS / PROGRESS_STEPS
    }
}

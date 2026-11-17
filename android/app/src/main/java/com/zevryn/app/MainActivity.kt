package com.zevryn.app

import android.app.Activity
import android.os.Bundle
import android.util.Log
import android.widget.TextView

class MainActivity : Activity() {
    companion object {
        private const val TAG = "Zevryn"

        init {
            try {
                System.loadLibrary("zevryn")
                Log.i(TAG, "libzevryn.so loaded")
            } catch (t: Throwable) {
                Log.e(TAG, "Failed to load libzevryn.so", t)
            }
        }
    }

    private external fun coreVersion(): String
    private external fun coreStatus(): String

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        Log.i(TAG, "onCreate")

        val text = try {
            "Zevryn\n${coreVersion()}\n${coreStatus()}"
        } catch (t: Throwable) {
            Log.e(TAG, "Native call failed", t)
            "Zevryn\n(native call failed - see logcat)"
        }

        val tv = TextView(this).apply {
            this.text = text
            textSize = 20f
            setPadding(48, 48, 48, 48)
        }
        setContentView(tv)
    }
}

package com.zevryn.app

import android.os.Bundle
import android.util.Log
import androidx.appcompat.app.AppCompatActivity
import androidx.fragment.app.Fragment
import androidx.fragment.app.FragmentActivity
import androidx.viewpager2.adapter.FragmentStateAdapter
import androidx.viewpager2.widget.ViewPager2
import com.google.android.material.tabs.TabLayout
import com.google.android.material.tabs.TabLayoutMediator

/**
 * Host activity for the Zevryn launcher UI.
 *
 * IMPORTANT: the package and class name must stay `com.zevryn.app.MainActivity`
 * because the Rust `#[no_mangle]` exports are named
 * `Java_com_zevryn_app_MainActivity_*` and resolve against these instance
 * methods. Every native call is guarded so a missing/failed library never
 * prevents the launcher from opening.
 */
class MainActivity : AppCompatActivity() {
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

    // ---- JNI declarations (instance methods — keep names/signatures exact) --
    private external fun coreVersionCode(): Int
    private external fun coreModuleCount(): Int
    private external fun nativeDashboardCards(): String
    private external fun nativeGameLibrary(): String

    // ---- Guarded accessors consumed by the fragments ------------------------
    fun coreVersion(): Int = try {
        coreVersionCode()
    } catch (t: Throwable) {
        Log.e(TAG, "coreVersionCode failed", t)
        -1
    }

    fun coreModules(): Int = try {
        coreModuleCount()
    } catch (t: Throwable) {
        Log.e(TAG, "coreModuleCount failed", t)
        -1
    }

    fun dashboardCardsJson(): String = try {
        nativeDashboardCards()
    } catch (t: Throwable) {
        Log.e(TAG, "nativeDashboardCards failed", t)
        "[]"
    }

    fun gameLibraryJson(): String = try {
        nativeGameLibrary()
    } catch (t: Throwable) {
        Log.e(TAG, "nativeGameLibrary failed", t)
        "[]"
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        Log.i(TAG, "onCreate — core v${coreVersion()}, modules: ${coreModules()}")
        setContentView(R.layout.activity_main)

        val viewPager = findViewById<ViewPager2>(R.id.page_container)
        val tabLayout = findViewById<TabLayout>(R.id.main_tab_view)

        viewPager.adapter = MainTabsAdapter(this)
        viewPager.offscreenPageLimit = 1

        TabLayoutMediator(tabLayout, viewPager) { tab, position ->
            tab.text = when (position) {
                0 -> getString(R.string.tab_dashboard)
                else -> getString(R.string.tab_instant_play)
            }
        }.attach()
    }

    /** Two tabs: Dashboard (cards) and Instant Play (game library). */
    private class MainTabsAdapter(activity: FragmentActivity) : FragmentStateAdapter(activity) {
        override fun getItemCount(): Int = 2

        override fun createFragment(position: Int): Fragment =
            if (position == 0) DashboardFragment() else LibraryFragment()
    }
}

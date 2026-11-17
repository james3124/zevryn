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
import com.zevryn.app.screens.DeviceFragment
import com.zevryn.app.screens.GameDetailFragment
import com.zevryn.app.screens.SearchFragment
import com.zevryn.app.screens.SettingsFragment
import com.zevryn.app.screens.controller.ButtonMappingFragment
import com.zevryn.app.screens.controller.GamepadFragment
import com.zevryn.app.screens.media.AlbumFragment
import com.zevryn.app.screens.media.FeedbackFragment
import com.zevryn.app.screens.media.NotificationFragment
import com.zevryn.app.screens.misc.AboutFragment
import com.zevryn.app.screens.misc.MobileGameFragment
import com.zevryn.app.screens.misc.VirtualButtonsFragment

/**
 * Host activity for the Zevryn launcher UI.
 *
 * Package/class name must stay `com.zevryn.app.MainActivity`: the Rust
 * #[no_mangle] exports are `Java_com_zevryn_app_MainActivity_*`.
 * Every native call is guarded so a failed library never blocks opening.
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

    // ---- JNI declarations (instance methods — names/signatures exact) --------
    private external fun coreVersionCode(): Int
    private external fun coreModuleCount(): Int
    private external fun nativeDashboardCards(): String
    private external fun nativeGameLibrary(): String

    // ---- Guarded accessors consumed by fragments ----------------------------
    fun coreVersion(): Int = try { coreVersionCode() } catch (t: Throwable) { Log.e(TAG, "coreVersionCode", t); -1 }
    fun coreModules(): Int = try { coreModuleCount() } catch (t: Throwable) { Log.e(TAG, "coreModuleCount", t); -1 }
    fun dashboardCardsJson(): String = try { nativeDashboardCards() } catch (t: Throwable) { Log.e(TAG, "nativeDashboardCards", t); "[]" }
    fun gameLibraryJson(): String = try { nativeGameLibrary() } catch (t: Throwable) { Log.e(TAG, "nativeGameLibrary", t); "[]" }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        Log.i(TAG, "onCreate — core v${coreVersion()}, modules: ${coreModules()}")
        setContentView(R.layout.activity_main)

        val viewPager = findViewById<ViewPager2>(R.id.page_container)
        val tabLayout = findViewById<TabLayout>(R.id.main_tab_view)

        viewPager.adapter = MainTabsAdapter(this)
        viewPager.offscreenPageLimit = 1

        val labels = tabLabels()
        TabLayoutMediator(tabLayout, viewPager) { tab, position ->
            tab.text = labels.getOrNull(position) ?: "Tab ${position + 1}"
        }.attach()
    }

    private fun tabLabels(): List<String> = listOf(
        getString(R.string.tab_dashboard),      // 0
        getString(R.string.tab_instant_play),   // 1
        getString(R.string.tab_search),         // 2
        getString(R.string.tab_game_detail),    // 3
        getString(R.string.tab_settings),       // 4
        getString(R.string.tab_device),         // 5
        getString(R.string.tab_gamepad),        // 6
        getString(R.string.tab_mapped),         // 7
        getString(R.string.media_tab_album),    // 8
        getString(R.string.media_tab_notification), // 9
        getString(R.string.media_tab_feedback), // 10
        getString(R.string.misc_tab_mobile_game), // 11
        getString(R.string.misc_tab_virtual_buttons), // 12
        getString(R.string.misc_tab_about),     // 13
    )

    private class MainTabsAdapter(activity: FragmentActivity) : FragmentStateAdapter(activity) {
        override fun getItemCount(): Int = 14

        override fun createFragment(position: Int): Fragment = when (position) {
            0 -> DashboardFragment()
            1 -> LibraryFragment()
            2 -> SearchFragment()
            3 -> GameDetailFragment()
            4 -> SettingsFragment()
            5 -> DeviceFragment()
            6 -> GamepadFragment()
            7 -> ButtonMappingFragment()
            8 -> AlbumFragment()
            9 -> NotificationFragment()
            10 -> FeedbackFragment()
            11 -> MobileGameFragment()
            12 -> VirtualButtonsFragment()
            13 -> AboutFragment()
            else -> DashboardFragment()
        }
    }
}

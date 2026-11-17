package com.zevryn.app.screens

import android.graphics.Color
import android.graphics.drawable.GradientDrawable
import android.os.Bundle
import android.util.Log
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.Button
import android.widget.TextView
import android.widget.Toast
import androidx.fragment.app.Fragment
import com.google.android.material.dialog.MaterialAlertDialogBuilder
import com.zevryn.app.R

/**
 * Game detail screen for the Zevryn launcher.
 *
 * Accepts a game through its arguments bundle (id / title / kind) with safe
 * defaults, so it never crashes when opened without arguments. Everything on
 * this screen is local: no native/JNI calls are made, so it is safe even if the
 * Rust library failed to load.
 *
 * The long-press action menu mirrors the GameHub Lite game-detail actions that
 * the Rust core models (PC game settings, compatibility check, library
 * add/remove, uninstall). account / friends / cloud actions are deliberately
 * omitted from the UI.
 */
class GameDetailFragment : Fragment() {

    private var inLibrary = false

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View = inflater.inflate(R.layout.fragment_game_detail, container, false)

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        super.onViewCreated(view, savedInstanceState)

        val args = arguments
        val title = args?.getString(ARG_TITLE)?.takeIf { it.isNotBlank() }
            ?: getString(R.string.zd_detail_default_title)
        val kind = args?.getString(ARG_KIND)?.takeIf { it.isNotBlank() }
            ?: getString(R.string.unknown_kind)
        // `id` is accepted for parity with the library/card arguments but is
        // not needed by this offline screen.

        val banner = view.findViewById<View>(R.id.detailBanner)
        val kindChip = view.findViewById<TextView>(R.id.detailKind)
        val titleView = view.findViewById<TextView>(R.id.detailTitle)
        val description = view.findViewById<TextView>(R.id.detailDescription)
        val moreText = view.findViewById<TextView>(R.id.moreDetailsText)
        val moreToggle = view.findViewById<Button>(R.id.moreDetailsToggle)
        val playButton = view.findViewById<Button>(R.id.playNowButton)

        titleView.text = title
        kindChip.text = kind.uppercase()
        description.text = getString(R.string.zd_detail_description)
        moreText.text = getString(R.string.zd_detail_more_text)

        // Colored placeholder cover, tinted by game kind (no image assets/native).
        banner.background = GradientDrawable().apply {
            shape = GradientDrawable.RECTANGLE
            setColor(colorForKind(kind))
        }

        playButton.setOnClickListener {
            toast(getString(R.string.zd_detail_play_toast, title, startTypeFor(kind)))
        }

        moreToggle.setOnClickListener {
            val expanding = moreText.visibility != View.VISIBLE
            moreText.visibility = if (expanding) View.VISIBLE else View.GONE
            moreToggle.text = getString(
                if (expanding) R.string.zd_detail_less_info else R.string.zd_detail_more_info
            )
        }

        // Long-press the cover (or anywhere on the detail page) for the menu.
        val showMenu = View.OnLongClickListener {
            showActionsDialog(title)
            true
        }
        view.findViewById<View>(R.id.detailBannerContainer).setOnLongClickListener(showMenu)
        view.findViewById<View>(R.id.detailRoot).setOnLongClickListener(showMenu)
    }

    private fun colorForKind(kind: String): Int = try {
        Color.parseColor(
            when (kind.uppercase()) {
                "PC", "STEAM" -> "#1E88E5"
                "MOBILE", "ANDROID" -> "#43A047"
                "CLOUD" -> "#8E24AA"
                else -> "#FF2D6C"
            }
        )
    } catch (t: Throwable) {
        Log.e("Zevryn", "Invalid game detail banner color", t)
        Color.parseColor("#FF2D6C")
    }

    /**
     * Start-type codes modeled by the Rust core. Reported in the demo toast so
     * the UI stays faithful without invoking any native launch path.
     */
    private fun startTypeFor(kind: String): Int = when (kind.uppercase()) {
        "STEAM" -> START_STEAM
        "CLOUD" -> START_CLOUD
        else -> START_PC_EMU
    }

    private fun showActionsDialog(title: String) {
        val addRemoveLabel = getString(
            if (inLibrary) R.string.zd_detail_remove_lib else R.string.zd_detail_add_lib
        )
        val items = arrayOf<CharSequence>(
            getString(R.string.zd_detail_pc_game_settings),
            getString(R.string.zd_detail_check_compat),
            addRemoveLabel,
            getString(R.string.zd_detail_uninstall)
        )
        MaterialAlertDialogBuilder(requireContext())
            .setTitle(getString(R.string.zd_detail_menu_title, title))
            .setItems(items) { _, which ->
                when (which) {
                    0 -> toast(getString(R.string.zd_detail_action_toast, title, items[0]))
                    1 -> toast(getString(R.string.zd_detail_action_toast, title, items[1]))
                    2 -> {
                        inLibrary = !inLibrary
                        val message = if (inLibrary) {
                            R.string.zd_detail_added_toast
                        } else {
                            R.string.zd_detail_removed_toast
                        }
                        toast(getString(message, title))
                    }
                    3 -> confirmUninstall(title)
                }
            }
            .setNegativeButton(android.R.string.cancel, null)
            .show()
    }

    private fun confirmUninstall(title: String) {
        MaterialAlertDialogBuilder(requireContext())
            .setTitle(getString(R.string.zd_detail_uninstall))
            .setMessage(getString(R.string.zd_detail_uninstall_confirm, title))
            .setPositiveButton(android.R.string.ok) { _, _ ->
                toast(getString(R.string.zd_detail_uninstall_done, title))
            }
            .setNegativeButton(android.R.string.cancel, null)
            .show()
    }

    private fun toast(message: String) {
        Toast.makeText(requireContext(), message, Toast.LENGTH_SHORT).show()
    }

    companion object {
        private const val ARG_ID = "id"
        private const val ARG_TITLE = "title"
        private const val ARG_KIND = "kind"

        // Rust core start-type codes (no native call here; used for display only).
        private const val START_PC_EMU = 1403
        private const val START_STEAM = 1407
        private const val START_CLOUD = 1406

        fun newInstance(id: String, title: String, kind: String): GameDetailFragment =
            GameDetailFragment().apply {
                arguments = Bundle().apply {
                    putString(ARG_ID, id)
                    putString(ARG_TITLE, title)
                    putString(ARG_KIND, kind)
                }
            }
    }
}

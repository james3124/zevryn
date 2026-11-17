package com.zevryn.app.screens

import android.content.Context
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import android.widget.Toast
import androidx.appcompat.widget.SwitchCompat
import androidx.fragment.app.Fragment
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import com.zevryn.app.FocusHelper
import com.zevryn.app.R

/**
 * "Settings" screen for the Zevryn launcher.
 *
 * Scope is intentionally offline-only: offline mode, importing a game,
 * gamepad / key mapping and an About row. No account, cloud or telemetry.
 * No native/JNI calls are made here, so the screen is safe even if the Rust
 * library failed to load.
 */
class SettingsFragment : Fragment() {

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View = inflater.inflate(R.layout.fragment_settings, container, false)

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        super.onViewCreated(view, savedInstanceState)

        val prefs = requireContext().getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        val offlineOn = prefs.getBoolean(KEY_OFFLINE, true)

        val rows = listOf(
            SettingRow(
                title = getString(R.string.setting_offline_title),
                subtitle = getString(R.string.setting_offline_subtitle),
                kind = RowKind.TOGGLE,
                checked = offlineOn
            ),
            SettingRow(
                title = getString(R.string.setting_import_title),
                subtitle = getString(R.string.setting_import_subtitle),
                kind = RowKind.IMPORT
            ),
            SettingRow(
                title = getString(R.string.setting_keymap_title),
                subtitle = getString(R.string.setting_keymap_subtitle),
                kind = RowKind.KEYMAP
            ),
            SettingRow(
                title = getString(R.string.setting_about_title),
                subtitle = getString(R.string.setting_about_subtitle),
                kind = RowKind.ABOUT
            )
        )

        val recycler = view.findViewById<RecyclerView>(R.id.settingsRecycler)
        recycler.layoutManager = LinearLayoutManager(requireContext())
        recycler.adapter = SettingsAdapter(rows) { row, position ->
            when (row.kind) {
                RowKind.TOGGLE -> {
                    val next = !row.checked
                    row.checked = next
                    prefs.edit().putBoolean(KEY_OFFLINE, next).apply()
                    recycler.adapter?.notifyItemChanged(position)
                    toast(
                        getString(
                            if (next) R.string.setting_offline_on
                            else R.string.setting_offline_off
                        )
                    )
                }
                RowKind.IMPORT -> toast(getString(R.string.setting_import_toast))
                RowKind.KEYMAP -> toast(getString(R.string.setting_keymap_toast))
                RowKind.ABOUT -> toast(row.subtitle) // Shows the static version string.
            }
        }
    }

    private fun toast(message: String) {
        Toast.makeText(requireContext(), message, Toast.LENGTH_SHORT).show()
    }

    enum class RowKind { TOGGLE, IMPORT, KEYMAP, ABOUT }

    data class SettingRow(
        val title: String,
        val subtitle: String,
        val kind: RowKind,
        var checked: Boolean = false
    )

    class SettingsAdapter(
        private val rows: List<SettingRow>,
        private val onClick: (SettingRow, Int) -> Unit
    ) : RecyclerView.Adapter<SettingsAdapter.RowViewHolder>() {

        override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): RowViewHolder {
            val view = LayoutInflater.from(parent.context)
                .inflate(R.layout.item_setting_row, parent, false)
            return RowViewHolder(view)
        }

        override fun getItemCount(): Int = rows.size

        override fun onBindViewHolder(holder: RowViewHolder, position: Int) {
            holder.bind(rows[position], position, itemCount, onClick)
        }

        class RowViewHolder(itemView: View) : RecyclerView.ViewHolder(itemView) {
            private val title: TextView = itemView.findViewById(R.id.settingTitle)
            private val subtitle: TextView = itemView.findViewById(R.id.settingSubtitle)
            private val toggle: SwitchCompat = itemView.findViewById(R.id.settingSwitch)

            fun bind(
                row: SettingRow,
                position: Int,
                count: Int,
                onClick: (SettingRow, Int) -> Unit
            ) {
                title.text = row.title
                subtitle.text = row.subtitle
                toggle.visibility = if (row.kind == RowKind.TOGGLE) View.VISIBLE else View.GONE
                toggle.isChecked = row.checked

                itemView.setOnClickListener { onClick(row, position) }
                // Single-column list: wire up/down neighbours for D-pad navigation.
                FocusHelper.wireGridFocus(itemView, position, count, 1)
            }
        }
    }

    companion object {
        private const val PREFS_NAME = "zevryn_settings"
        private const val KEY_OFFLINE = "offline_mode"
    }
}

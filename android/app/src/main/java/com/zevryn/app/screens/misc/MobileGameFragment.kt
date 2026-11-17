package com.zevryn.app.screens.misc

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import android.widget.Toast
import androidx.fragment.app.Fragment
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import com.zevryn.app.FocusHelper
import com.zevryn.app.R

/**
 * "Mobile Games" screen for the Zevryn launcher — an offline-safe take on a
 * mobile-games tab (grounded in GameHub Lite's MobileGameV4Activity).
 *
 * The list is static local demo data and tapping a row only shows a Toast.
 * There are no native/JNI calls and no package-manager queries, so the screen
 * never crashes even if the Rust library failed to load.
 */
class MobileGameFragment : Fragment() {

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View = inflater.inflate(R.layout.misc_mobile_game, container, false)

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        super.onViewCreated(view, savedInstanceState)

        val games = listOf(
            game(R.string.misc_game_1_badge, R.string.misc_game_1_name, R.string.misc_game_1_meta),
            game(R.string.misc_game_2_badge, R.string.misc_game_2_name, R.string.misc_game_2_meta),
            game(R.string.misc_game_3_badge, R.string.misc_game_3_name, R.string.misc_game_3_meta),
            game(R.string.misc_game_4_badge, R.string.misc_game_4_name, R.string.misc_game_4_meta),
            game(R.string.misc_game_5_badge, R.string.misc_game_5_name, R.string.misc_game_5_meta)
        )

        val recycler = view.findViewById<RecyclerView>(R.id.miscGameRecycler)
        recycler.layoutManager = LinearLayoutManager(requireContext())
        recycler.adapter = MobileGameAdapter(games) { game ->
            Toast.makeText(
                requireContext(),
                getString(R.string.misc_game_selected_toast, game.name),
                Toast.LENGTH_SHORT
            ).show()
        }
    }

    private fun game(badgeRes: Int, nameRes: Int, metaRes: Int): MobileGame =
        MobileGame(getString(badgeRes), getString(nameRes), getString(metaRes))

    private data class MobileGame(val badge: String, val name: String, val meta: String)

    private class MobileGameAdapter(
        private val games: List<MobileGame>,
        private val onClick: (MobileGame) -> Unit
    ) : RecyclerView.Adapter<MobileGameAdapter.GameViewHolder>() {

        override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): GameViewHolder {
            val view = LayoutInflater.from(parent.context)
                .inflate(R.layout.misc_item_game, parent, false)
            return GameViewHolder(view)
        }

        override fun getItemCount(): Int = games.size

        override fun onBindViewHolder(holder: GameViewHolder, position: Int) {
            holder.bind(games[position], position, itemCount, onClick)
        }

        class GameViewHolder(itemView: View) : RecyclerView.ViewHolder(itemView) {
            private val badge: TextView = itemView.findViewById(R.id.miscGameBadge)
            private val name: TextView = itemView.findViewById(R.id.miscGameName)
            private val meta: TextView = itemView.findViewById(R.id.miscGameMeta)

            fun bind(
                game: MobileGame,
                position: Int,
                count: Int,
                onClick: (MobileGame) -> Unit
            ) {
                badge.text = game.badge
                name.text = game.name
                meta.text = game.meta
                itemView.setOnClickListener { onClick(game) }
                // Single-column list: wire up/down neighbours for D-pad navigation.
                FocusHelper.wireGridFocus(itemView, position, count, 1)
            }
        }
    }
}

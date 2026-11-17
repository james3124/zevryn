package com.zevryn.app

import android.graphics.Color
import android.graphics.drawable.GradientDrawable
import android.util.Log
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import android.widget.Toast
import androidx.recyclerview.widget.RecyclerView

/** One game row parsed from nativeGameLibrary(). */
data class LibraryGame(val id: String, val title: String, val kind: String)

class LibraryGameAdapter(
    private var games: List<LibraryGame>
) : RecyclerView.Adapter<LibraryGameAdapter.GameViewHolder>() {

    override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): GameViewHolder {
        val view = LayoutInflater.from(parent.context)
            .inflate(R.layout.item_library_game, parent, false)
        return GameViewHolder(view)
    }

    override fun getItemCount(): Int = games.size

    override fun onBindViewHolder(holder: GameViewHolder, position: Int) {
        holder.bind(games[position], position, itemCount, COLUMNS)
    }

    fun submitList(list: List<LibraryGame>) {
        games = list
        notifyDataSetChanged()
    }

    class GameViewHolder(itemView: View) : RecyclerView.ViewHolder(itemView) {
        private val badge: TextView = itemView.findViewById(R.id.kindBadge)
        private val title: TextView = itemView.findViewById(R.id.gameTitle)
        private val kind: TextView = itemView.findViewById(R.id.gameKind)

        fun bind(game: LibraryGame, position: Int, count: Int, columns: Int) {
            title.text = game.title
            kind.text = game.kind.ifEmpty { itemView.context.getString(R.string.unknown_kind) }
            badge.text = game.kind.take(2).ifEmpty { "?" }.uppercase()

            val color = try {
                Color.parseColor(
                    when (game.kind.uppercase()) {
                        "PC" -> "#1E88E5"
                        "MOBILE" -> "#43A047"
                        else -> "#546E7A"
                    }
                )
            } catch (t: Throwable) {
                Log.e("Zevryn", "Badge color parse failed", t)
                Color.parseColor("#546E7A")
            }
            badge.background = GradientDrawable().apply {
                shape = GradientDrawable.RECTANGLE
                cornerRadius = 10f
                setColor(color)
            }

            itemView.setOnClickListener {
                Toast.makeText(
                    itemView.context,
                    itemView.context.getString(R.string.game_click_toast, game.title),
                    Toast.LENGTH_SHORT
                ).show()
            }

            FocusHelper.wireGridFocus(itemView, position, count, columns)
        }
    }

    companion object {
        private const val COLUMNS = 3
    }
}

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

/** One dashboard tile parsed from nativeDashboardCards(). */
data class DashboardCard(val title: String, val kind: String, val color: String)

class DashboardCardAdapter(
    private var cards: List<DashboardCard>
) : RecyclerView.Adapter<DashboardCardAdapter.CardViewHolder>() {

    override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): CardViewHolder {
        val view = LayoutInflater.from(parent.context)
            .inflate(R.layout.item_game_card, parent, false)
        return CardViewHolder(view)
    }

    override fun getItemCount(): Int = cards.size

    override fun onBindViewHolder(holder: CardViewHolder, position: Int) {
        holder.bind(cards[position], position, itemCount, COLUMNS)
    }

    fun submitList(list: List<DashboardCard>) {
        cards = list
        notifyDataSetChanged()
    }

    class CardViewHolder(itemView: View) : RecyclerView.ViewHolder(itemView) {
        private val banner: View = itemView.findViewById(R.id.cardBanner)
        private val title: TextView = itemView.findViewById(R.id.cardTitle)
        private val kind: TextView = itemView.findViewById(R.id.cardKind)

        fun bind(card: DashboardCard, position: Int, count: Int, columns: Int) {
            title.text = card.title
            kind.text = card.kind

            val color = try {
                Color.parseColor(card.color)
            } catch (t: Throwable) {
                Log.e("Zevryn", "Invalid card color: ${card.color}", t)
                Color.parseColor("#FF2D6C")
            }
            banner.background = GradientDrawable().apply {
                shape = GradientDrawable.RECTANGLE
                cornerRadius = 12f
                setColor(color)
            }

            itemView.setOnClickListener {
                Toast.makeText(
                    itemView.context,
                    itemView.context.getString(R.string.card_click_toast, card.title),
                    Toast.LENGTH_SHORT
                ).show()
            }

            FocusHelper.wireGridFocus(itemView, position, count, columns)
        }
    }

    companion object {
        private const val COLUMNS = 2
    }
}

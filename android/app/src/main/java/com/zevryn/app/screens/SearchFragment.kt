package com.zevryn.app.screens

import android.graphics.Color
import android.graphics.drawable.GradientDrawable
import android.os.Bundle
import android.text.Editable
import android.text.TextWatcher
import android.util.Log
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.AdapterView
import android.widget.EditText
import android.widget.Spinner
import android.widget.TextView
import android.widget.Toast
import androidx.fragment.app.Fragment
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import com.zevryn.app.FocusHelper
import com.zevryn.app.R

/** One locally-defined search result. */
data class SearchGame(val id: String, val title: String, val kind: String)

/**
 * Search screen: an [EditText] plus a filter [Spinner] (All / PC / Mobile,
 * reusing the existing `filter_options` array) driving a locally-filtered list.
 * All data is hardcoded demo content; no native calls are made.
 */
class SearchFragment : Fragment() {

    private val catalogue = listOf(
        SearchGame("1", "Hollow Knight", "PC"),
        SearchGame("2", "Genshin Impact", "MOBILE"),
        SearchGame("3", "Elden Ring", "PC"),
        SearchGame("4", "Call of Duty: Mobile", "MOBILE"),
        SearchGame("5", "Stardew Valley", "PC"),
        SearchGame("6", "Honkai: Star Rail", "MOBILE"),
        SearchGame("7", "Cyberpunk 2077", "PC"),
        SearchGame("8", "Fortnite", "CLOUD")
    )

    private var query = ""
    private var filterMode = 0 // 0: All, 1: PC, 2: Mobile
    private lateinit var adapter: SearchAdapter

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View = inflater.inflate(R.layout.fragment_search, container, false)

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        super.onViewCreated(view, savedInstanceState)

        val results = view.findViewById<RecyclerView>(R.id.searchResults)
        results.layoutManager = LinearLayoutManager(requireContext())
        adapter = SearchAdapter(emptyList())
        results.adapter = adapter

        val input = view.findViewById<EditText>(R.id.searchInput)
        input.addTextChangedListener(object : TextWatcher {
            override fun beforeTextChanged(s: CharSequence?, start: Int, count: Int, after: Int) = Unit

            override fun onTextChanged(s: CharSequence?, start: Int, before: Int, count: Int) {
                query = s?.toString().orEmpty()
                applyFilters()
            }

            override fun afterTextChanged(s: Editable?) = Unit
        })

        view.findViewById<Spinner>(R.id.searchFilterSpinner).onItemSelectedListener =
            object : AdapterView.OnItemSelectedListener {
                override fun onItemSelected(
                    parent: AdapterView<*>, selected: View?, position: Int, id: Long
                ) {
                    filterMode = position
                    applyFilters()
                }

                override fun onNothingSelected(parent: AdapterView<*>) = Unit
            }

        applyFilters()
    }

    private fun applyFilters() {
        val q = query.trim()
        val result = catalogue.filter { game ->
            val matchesText = q.isEmpty() || game.title.contains(q, ignoreCase = true)
            val matchesKind = when (filterMode) {
                1 -> game.kind.equals("PC", ignoreCase = true)
                2 -> game.kind.equals("MOBILE", ignoreCase = true)
                else -> true
            }
            matchesText && matchesKind
        }
        adapter.submitList(result)
    }

    private class SearchAdapter(
        private var games: List<SearchGame>
    ) : RecyclerView.Adapter<SearchAdapter.ResultViewHolder>() {

        override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): ResultViewHolder {
            val view = LayoutInflater.from(parent.context)
                .inflate(R.layout.item_search_result, parent, false)
            return ResultViewHolder(view)
        }

        override fun getItemCount(): Int = games.size

        override fun onBindViewHolder(holder: ResultViewHolder, position: Int) {
            holder.bind(games[position], position, itemCount)
        }

        fun submitList(list: List<SearchGame>) {
            games = list
            notifyDataSetChanged()
        }

        class ResultViewHolder(itemView: View) : RecyclerView.ViewHolder(itemView) {
            private val badge: TextView = itemView.findViewById(R.id.resultBadge)
            private val title: TextView = itemView.findViewById(R.id.resultTitle)
            private val kind: TextView = itemView.findViewById(R.id.resultKind)

            fun bind(game: SearchGame, position: Int, count: Int) {
                title.text = game.title
                kind.text = game.kind.ifEmpty {
                    itemView.context.getString(R.string.unknown_kind)
                }
                badge.text = game.kind.take(2).ifEmpty { "?" }.uppercase()
                badge.background = GradientDrawable().apply {
                    shape = GradientDrawable.RECTANGLE
                    setColor(badgeColor(game.kind))
                }

                itemView.setOnClickListener {
                    Toast.makeText(
                        itemView.context,
                        itemView.context.getString(R.string.game_click_toast, game.title),
                        Toast.LENGTH_SHORT
                    ).show()
                }

                // Single-column list: D-pad focus walks up/down the results.
                FocusHelper.wireGridFocus(itemView, position, count, 1)
            }

            private fun badgeColor(kind: String): Int = try {
                Color.parseColor(
                    when (kind.uppercase()) {
                        "PC" -> "#1E88E5"
                        "MOBILE" -> "#43A047"
                        "CLOUD" -> "#8E24AA"
                        else -> "#546E7A"
                    }
                )
            } catch (t: Throwable) {
                Log.e("Zevryn", "Search badge color parse failed", t)
                Color.parseColor("#546E7A")
            }
        }
    }
}

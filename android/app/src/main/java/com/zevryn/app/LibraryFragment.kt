package com.zevryn.app

import android.os.Bundle
import android.util.Log
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.AdapterView
import android.widget.Spinner
import androidx.fragment.app.Fragment
import androidx.recyclerview.widget.GridLayoutManager
import androidx.recyclerview.widget.RecyclerView
import org.json.JSONArray

/** "Instant Play" tab — sortable/filterable game grid fed by nativeGameLibrary(). */
class LibraryFragment : Fragment() {

    private var allGames: List<LibraryGame> = emptyList()
    private lateinit var adapter: LibraryGameAdapter
    private var sortMode = 0 // 0: Title A-Z, 1: Title Z-A, 2: Platform
    private var filterMode = 0 // 0: All, 1: PC, 2: Mobile

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View = inflater.inflate(R.layout.fragment_library, container, false)

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        super.onViewCreated(view, savedInstanceState)
        val host = activity as? MainActivity
        allGames = parseGames(host?.gameLibraryJson())

        val grid = view.findViewById<RecyclerView>(R.id.libraryRecycler)
        grid.layoutManager = GridLayoutManager(requireContext(), LIBRARY_COLUMNS)
        adapter = LibraryGameAdapter(emptyList())
        grid.adapter = adapter

        val sortSpinner = view.findViewById<Spinner>(R.id.sortSpinner)
        val filterSpinner = view.findViewById<Spinner>(R.id.filterSpinner)
        sortSpinner.onItemSelectedListener = simpleItemSelected { sortMode = it; applyFilters() }
        filterSpinner.onItemSelectedListener = simpleItemSelected { filterMode = it; applyFilters() }

        applyFilters()
    }

    private fun applyFilters() {
        var result = when (filterMode) {
            1 -> allGames.filter { it.kind.equals("PC", ignoreCase = true) }
            2 -> allGames.filter { it.kind.equals("MOBILE", ignoreCase = true) }
            else -> allGames
        }
        result = when (sortMode) {
            0 -> result.sortedBy { it.title.lowercase() }
            1 -> result.sortedByDescending { it.title.lowercase() }
            2 -> result.sortedBy { it.kind.lowercase() }
            else -> result
        }
        adapter.submitList(result)
    }

    private fun parseGames(json: String?): List<LibraryGame> {
        if (json.isNullOrBlank()) return emptyList()
        return try {
            val array = JSONArray(json)
            (0 until array.length()).mapNotNull { i ->
                val obj = array.optJSONObject(i) ?: return@mapNotNull null
                LibraryGame(
                    id = obj.optString("id", ""),
                    title = obj.optString("title", ""),
                    kind = obj.optString("kind", "")
                )
            }
        } catch (t: Throwable) {
            Log.e("Zevryn", "Failed to parse game library JSON", t)
            emptyList()
        }
    }

    private fun simpleItemSelected(onSelected: (Int) -> Unit) =
        object : AdapterView.OnItemSelectedListener {
            override fun onItemSelected(
                parent: AdapterView<*>, view: View?, position: Int, id: Long
            ) = onSelected(position)

            override fun onNothingSelected(parent: AdapterView<*>) = Unit
        }

    companion object {
        private const val LIBRARY_COLUMNS = 3
    }
}

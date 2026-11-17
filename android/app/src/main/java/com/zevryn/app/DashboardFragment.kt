package com.zevryn.app

import android.os.Bundle
import android.util.Log
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import androidx.fragment.app.Fragment
import androidx.recyclerview.widget.GridLayoutManager
import androidx.recyclerview.widget.RecyclerView
import org.json.JSONArray

/** "Dashboard" tab — focusable card grid fed by nativeDashboardCards(). */
class DashboardFragment : Fragment() {

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View = inflater.inflate(R.layout.fragment_dashboard, container, false)

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        super.onViewCreated(view, savedInstanceState)
        val host = activity as? MainActivity

        val coreInfo = view.findViewById<TextView>(R.id.coreInfo)
        coreInfo.text = getString(
            R.string.core_info,
            host?.coreVersion() ?: -1,
            host?.coreModules() ?: -1
        )

        val grid = view.findViewById<RecyclerView>(R.id.my_container_view)
        grid.layoutManager = GridLayoutManager(requireContext(), DASHBOARD_COLUMNS)
        grid.adapter = DashboardCardAdapter(parseCards(host?.dashboardCardsJson()))
    }

    private fun parseCards(json: String?): List<DashboardCard> {
        if (json.isNullOrBlank()) return emptyList()
        return try {
            val array = JSONArray(json)
            (0 until array.length()).mapNotNull { i ->
                val obj = array.optJSONObject(i) ?: return@mapNotNull null
                DashboardCard(
                    title = obj.optString("title", ""),
                    kind = obj.optString("kind", ""),
                    color = obj.optString("color", "#FF2D6C")
                )
            }
        } catch (t: Throwable) {
            Log.e("Zevryn", "Failed to parse dashboard cards JSON", t)
            emptyList()
        }
    }

    companion object {
        private const val DASHBOARD_COLUMNS = 2
    }
}

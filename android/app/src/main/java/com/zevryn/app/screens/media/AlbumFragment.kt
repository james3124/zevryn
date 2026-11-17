package com.zevryn.app.screens.media

import android.graphics.Color
import android.graphics.drawable.GradientDrawable
import android.os.Bundle
import android.util.TypedValue
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import android.widget.Toast
import androidx.fragment.app.Fragment
import androidx.recyclerview.widget.GridLayoutManager
import androidx.recyclerview.widget.RecyclerView
import com.zevryn.app.FocusHelper
import com.zevryn.app.R

/**
 * "Album" screen for the Zevryn launcher — a read-only grid of albums /
 * screenshots. Grounded in the reference album detail screen, but backed by
 * local demo data only: no account, cloud or native calls, so it cannot crash
 * even if the Rust library failed to load.
 */
class AlbumFragment : Fragment() {

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View = inflater.inflate(R.layout.media_album, container, false)

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        super.onViewCreated(view, savedInstanceState)

        val albums = listOf(
            Album(getString(R.string.media_album_1), "#FF2D6C"),
            Album(getString(R.string.media_album_2), "#1E88E5"),
            Album(getString(R.string.media_album_3), "#43A047"),
            Album(getString(R.string.media_album_4), "#8E24AA"),
            Album(getString(R.string.media_album_5), "#EF6C00"),
            Album(getString(R.string.media_album_6), "#546E7A")
        )

        val grid = view.findViewById<RecyclerView>(R.id.albumRecycler)
        grid.layoutManager = GridLayoutManager(requireContext(), COLUMNS)
        grid.adapter = AlbumAdapter(albums) { album ->
            Toast.makeText(
                requireContext(),
                getString(R.string.media_album_click_toast, album.title),
                Toast.LENGTH_SHORT
            ).show()
        }
    }

    private data class Album(val title: String, val bannerColor: String)

    private class AlbumAdapter(
        private val albums: List<Album>,
        private val onClick: (Album) -> Unit
    ) : RecyclerView.Adapter<AlbumAdapter.AlbumViewHolder>() {

        override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): AlbumViewHolder {
            val view = LayoutInflater.from(parent.context)
                .inflate(R.layout.media_item_album, parent, false)
            return AlbumViewHolder(view)
        }

        override fun getItemCount(): Int = albums.size

        override fun onBindViewHolder(holder: AlbumViewHolder, position: Int) {
            holder.bind(albums[position], position, itemCount, onClick)
        }

        class AlbumViewHolder(itemView: View) : RecyclerView.ViewHolder(itemView) {
            private val banner: View = itemView.findViewById(R.id.albumBanner)
            private val title: TextView = itemView.findViewById(R.id.albumTitle)

            fun bind(album: Album, position: Int, count: Int, onClick: (Album) -> Unit) {
                title.text = album.title

                // Colored placeholder banner (no image decoding / network).
                val radius = TypedValue.applyDimension(
                    TypedValue.COMPLEX_UNIT_DIP, 12f, itemView.resources.displayMetrics
                )
                val color = try {
                    Color.parseColor(album.bannerColor)
                } catch (t: Throwable) {
                    Color.parseColor("#FF2D6C")
                }
                banner.background = GradientDrawable().apply {
                    shape = GradientDrawable.RECTANGLE
                    cornerRadii = floatArrayOf(radius, radius, radius, radius, 0f, 0f, 0f, 0f)
                    setColor(color)
                }

                itemView.setOnClickListener { onClick(album) }
                FocusHelper.wireGridFocus(itemView, position, count, COLUMNS)
            }
        }

        companion object {
            private const val COLUMNS = 3
        }
    }

    companion object {
        private const val COLUMNS = 3
    }
}

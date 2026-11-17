package com.zevryn.app.screens.media

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
 * "Notifications" screen for the Zevryn launcher — a simple list of local
 * notifications that can be marked as read by tapping them. Grounded in the
 * reference notification list, but populated with hardcoded demo data and
 * with no account, cloud or native calls.
 */
class NotificationFragment : Fragment() {

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View = inflater.inflate(R.layout.media_notification, container, false)

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        super.onViewCreated(view, savedInstanceState)

        val notifications = listOf(
            AppNotification(
                getString(R.string.media_notification_1_title),
                getString(R.string.media_notification_1_msg),
                getString(R.string.media_notification_1_time)
            ),
            AppNotification(
                getString(R.string.media_notification_2_title),
                getString(R.string.media_notification_2_msg),
                getString(R.string.media_notification_2_time)
            ),
            AppNotification(
                getString(R.string.media_notification_3_title),
                getString(R.string.media_notification_3_msg),
                getString(R.string.media_notification_3_time)
            ),
            AppNotification(
                getString(R.string.media_notification_4_title),
                getString(R.string.media_notification_4_msg),
                getString(R.string.media_notification_4_time),
                read = true
            )
        )

        val recycler = view.findViewById<RecyclerView>(R.id.notificationRecycler)
        recycler.layoutManager = LinearLayoutManager(requireContext())
        var adapter: NotificationAdapter? = null
        adapter = NotificationAdapter(notifications) { notification, position ->
            if (!notification.read) {
                notification.read = true
                adapter?.notifyItemChanged(position)
                Toast.makeText(
                    requireContext(),
                    R.string.media_notification_marked_read,
                    Toast.LENGTH_SHORT
                ).show()
            }
        }
        recycler.adapter = adapter
    }

    private data class AppNotification(
        val title: String,
        val message: String,
        val time: String,
        var read: Boolean = false
    )

    private class NotificationAdapter(
        private val notifications: List<AppNotification>,
        private val onClick: (AppNotification, Int) -> Unit
    ) : RecyclerView.Adapter<NotificationAdapter.NotificationViewHolder>() {

        override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): NotificationViewHolder {
            val view = LayoutInflater.from(parent.context)
                .inflate(R.layout.media_item_notification, parent, false)
            return NotificationViewHolder(view)
        }

        override fun getItemCount(): Int = notifications.size

        override fun onBindViewHolder(holder: NotificationViewHolder, position: Int) {
            holder.bind(notifications[position], position, itemCount, onClick)
        }

        class NotificationViewHolder(itemView: View) : RecyclerView.ViewHolder(itemView) {
            private val unread: View = itemView.findViewById(R.id.notifUnread)
            private val title: TextView = itemView.findViewById(R.id.notifTitle)
            private val message: TextView = itemView.findViewById(R.id.notifMessage)
            private val time: TextView = itemView.findViewById(R.id.notifTime)

            fun bind(
                notification: AppNotification,
                position: Int,
                count: Int,
                onClick: (AppNotification, Int) -> Unit
            ) {
                title.text = notification.title
                message.text = notification.message
                time.text = notification.time
                unread.visibility = if (notification.read) View.INVISIBLE else View.VISIBLE
                itemView.alpha = if (notification.read) 0.55f else 1f

                itemView.setOnClickListener { onClick(notification, position) }
                // Single-column list: wire up/down neighbours for D-pad navigation.
                FocusHelper.wireGridFocus(itemView, position, count, 1)
            }
        }
    }
}

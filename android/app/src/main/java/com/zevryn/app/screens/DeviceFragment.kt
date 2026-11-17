package com.zevryn.app.screens

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.Button
import android.widget.TextView
import android.widget.Toast
import androidx.fragment.app.Fragment
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import com.zevryn.app.FocusHelper
import com.zevryn.app.R

/**
 * "Devices" screen for the Zevryn launcher — a simplified, offline-safe take
 * on a device manager: a few placeholder devices plus a Scan action.
 *
 * Mirrors the shape of a device/controller manager (name + status rows) but
 * performs no real discovery or native calls, so it never crashes.
 */
class DeviceFragment : Fragment() {

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View = inflater.inflate(R.layout.fragment_device, container, false)

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        super.onViewCreated(view, savedInstanceState)

        val devices = listOf(
            Device(getString(R.string.device_1_name), getString(R.string.device_1_status)),
            Device(getString(R.string.device_2_name), getString(R.string.device_2_status)),
            Device(getString(R.string.device_3_name), getString(R.string.device_3_status))
        )

        val recycler = view.findViewById<RecyclerView>(R.id.deviceRecycler)
        recycler.layoutManager = LinearLayoutManager(requireContext())
        recycler.adapter = DeviceAdapter(devices) { device ->
            Toast.makeText(
                requireContext(),
                getString(R.string.device_selected_toast, device.name),
                Toast.LENGTH_SHORT
            ).show()
        }

        view.findViewById<Button>(R.id.scanButton).setOnClickListener {
            Toast.makeText(
                requireContext(),
                R.string.device_scanning_toast,
                Toast.LENGTH_SHORT
            ).show()
        }
    }

    data class Device(val name: String, val status: String)

    class DeviceAdapter(
        private val devices: List<Device>,
        private val onClick: (Device) -> Unit
    ) : RecyclerView.Adapter<DeviceAdapter.DeviceViewHolder>() {

        override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): DeviceViewHolder {
            val view = LayoutInflater.from(parent.context)
                .inflate(R.layout.item_device, parent, false)
            return DeviceViewHolder(view)
        }

        override fun getItemCount(): Int = devices.size

        override fun onBindViewHolder(holder: DeviceViewHolder, position: Int) {
            holder.bind(devices[position], position, itemCount, onClick)
        }

        class DeviceViewHolder(itemView: View) : RecyclerView.ViewHolder(itemView) {
            private val name: TextView = itemView.findViewById(R.id.deviceName)
            private val status: TextView = itemView.findViewById(R.id.deviceStatus)

            fun bind(
                device: Device,
                position: Int,
                count: Int,
                onClick: (Device) -> Unit
            ) {
                name.text = device.name
                status.text = device.status
                itemView.setOnClickListener { onClick(device) }
                // Single-column list: wire up/down neighbours for D-pad navigation.
                FocusHelper.wireGridFocus(itemView, position, count, 1)
            }
        }
    }
}

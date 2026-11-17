package com.zevryn.app.screens.controller

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
 * "Button Mapping" screen for the Zevryn launcher — a simplified, offline-safe
 * take on a key-binding editor (cf. GameHub's ButtonSettingActivity).
 *
 * Rows show an action and its bound key. Tapping a row shows a "Press a key..."
 * toast; there is deliberately no real key capture and no native/JNI call, so
 * the screen is safe even if the Rust library failed to load.
 */
class ButtonMappingFragment : Fragment() {

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View = inflater.inflate(R.layout.ctrl_button_mapping, container, false)

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        super.onViewCreated(view, savedInstanceState)

        val bindings = demoBindings()
        val recycler = view.findViewById<RecyclerView>(R.id.ctrlBindingRecycler)
        recycler.layoutManager = LinearLayoutManager(requireContext())
        recycler.adapter = CtrlBindingAdapter(bindings) {
            Toast.makeText(
                requireContext(),
                R.string.ctrl_press_key_toast,
                Toast.LENGTH_SHORT
            ).show()
        }
    }

    companion object {
        /** Local demo data shared with [GamepadFragment]. */
        fun demoBindings(): List<CtrlBinding> = listOf(
            CtrlBinding(R.string.ctrl_action_move, R.string.ctrl_key_w),
            CtrlBinding(R.string.ctrl_action_jump, R.string.ctrl_key_space),
            CtrlBinding(R.string.ctrl_action_attack, R.string.ctrl_key_mouse_left),
            CtrlBinding(R.string.ctrl_action_sprint, R.string.ctrl_key_shift),
            CtrlBinding(R.string.ctrl_action_interact, R.string.ctrl_key_e),
            CtrlBinding(R.string.ctrl_action_pause, R.string.ctrl_key_esc)
        )
    }
}

/** One action and the string resource id of the key currently bound to it. */
internal data class CtrlBinding(val actionRes: Int, val keyRes: Int)

/** Adapter shared by the gamepad and button-mapping lists. */
internal class CtrlBindingAdapter(
    private val bindings: List<CtrlBinding>,
    private val onClick: (CtrlBinding) -> Unit
) : RecyclerView.Adapter<CtrlBindingAdapter.BindingViewHolder>() {

    override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): BindingViewHolder {
        val view = LayoutInflater.from(parent.context)
            .inflate(R.layout.ctrl_item_binding, parent, false)
        return BindingViewHolder(view)
    }

    override fun getItemCount(): Int = bindings.size

    override fun onBindViewHolder(holder: BindingViewHolder, position: Int) {
        holder.bind(bindings[position], position, itemCount, onClick)
    }

    class BindingViewHolder(itemView: View) : RecyclerView.ViewHolder(itemView) {
        private val action: TextView = itemView.findViewById(R.id.ctrlActionName)
        private val key: TextView = itemView.findViewById(R.id.ctrlBoundKey)

        fun bind(
            binding: CtrlBinding,
            position: Int,
            count: Int,
            onClick: (CtrlBinding) -> Unit
        ) {
            val res = itemView.resources
            action.text = res.getString(binding.actionRes)
            key.text = res.getString(R.string.ctrl_bound_key, res.getString(binding.keyRes))
            itemView.setOnClickListener { onClick(binding) }
            // Single-column list: wire up/down neighbours for D-pad navigation.
            FocusHelper.wireGridFocus(itemView, position, count, 1)
        }
    }
}

package com.zevryn.app.screens.misc

import android.graphics.drawable.GradientDrawable
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import android.widget.Toast
import androidx.appcompat.widget.SwitchCompat
import androidx.core.content.ContextCompat
import androidx.fragment.app.Fragment
import com.zevryn.app.R

/**
 * "Virtual Buttons" screen for the Zevryn launcher — a static preview of the
 * on-screen control overlay used by virtual-gamepad profiles (grounded in
 * GameHub Lite's pcvirtualbtn DisplayActivity / ControlsProfileListActivity).
 *
 * Every button is a focusable View; pressing one only shows a Toast. The
 * overlay can be shown/hidden with the switch. No native calls are made.
 */
class VirtualButtonsFragment : Fragment() {

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View = inflater.inflate(R.layout.misc_virtual_buttons, container, false)

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        super.onViewCreated(view, savedInstanceState)

        // Show / hide the on-screen overlay.
        val overlay = view.findViewById<View>(R.id.miscOverlayRoot)
        view.findViewById<SwitchCompat>(R.id.miscOverlaySwitch)
            .setOnCheckedChangeListener { _, checked ->
                overlay.visibility = if (checked) View.VISIBLE else View.GONE
            }

        // Every round button shares the same look and a Toast-only action.
        val buttons = listOf(
            R.id.miscDpadUp to R.string.misc_dpad_up,
            R.id.miscDpadDown to R.string.misc_dpad_down,
            R.id.miscDpadLeft to R.string.misc_dpad_left,
            R.id.miscDpadRight to R.string.misc_dpad_right,
            R.id.miscDpadCenter to R.string.misc_dpad_center,
            R.id.miscBtnA to R.string.misc_btn_a,
            R.id.miscBtnB to R.string.misc_btn_b,
            R.id.miscBtnX to R.string.misc_btn_x,
            R.id.miscBtnY to R.string.misc_btn_y
        )
        buttons.forEach { (viewId, labelRes) ->
            val button = view.findViewById<TextView>(viewId)
            button.background = ovalBackground()
            button.setOnClickListener {
                toast(getString(R.string.misc_virtual_btn_toast, getString(labelRes)))
            }
        }

        wireDpadFocus(view)
    }

    /** D-pad cross navigation plus a bridge into the A/B/X/Y cluster. */
    private fun wireDpadFocus(root: View) {
        val up = root.findViewById<View>(R.id.miscDpadUp)
        val down = root.findViewById<View>(R.id.miscDpadDown)
        val left = root.findViewById<View>(R.id.miscDpadLeft)
        val right = root.findViewById<View>(R.id.miscDpadRight)
        val center = root.findViewById<View>(R.id.miscDpadCenter)

        up.nextFocusDownId = R.id.miscDpadCenter
        center.nextFocusUpId = R.id.miscDpadUp
        center.nextFocusDownId = R.id.miscDpadDown
        center.nextFocusLeftId = R.id.miscDpadLeft
        center.nextFocusRightId = R.id.miscDpadRight
        left.nextFocusUpId = R.id.miscDpadUp
        left.nextFocusDownId = R.id.miscDpadDown
        right.nextFocusUpId = R.id.miscDpadUp
        right.nextFocusDownId = R.id.miscDpadDown
        down.nextFocusUpId = R.id.miscDpadCenter

        // Bridge the two clusters for gamepad users.
        right.nextFocusRightId = R.id.miscBtnX
        root.findViewById<View>(R.id.miscBtnX).nextFocusLeftId = R.id.miscDpadRight

        // A/B/X/Y form a small 2x2 grid.
        root.findViewById<View>(R.id.miscBtnX).nextFocusRightId = R.id.miscBtnY
        root.findViewById<View>(R.id.miscBtnX).nextFocusDownId = R.id.miscBtnA
        root.findViewById<View>(R.id.miscBtnY).nextFocusLeftId = R.id.miscBtnX
        root.findViewById<View>(R.id.miscBtnY).nextFocusDownId = R.id.miscBtnB
        root.findViewById<View>(R.id.miscBtnA).nextFocusUpId = R.id.miscBtnX
        root.findViewById<View>(R.id.miscBtnA).nextFocusRightId = R.id.miscBtnB
        root.findViewById<View>(R.id.miscBtnB).nextFocusUpId = R.id.miscBtnY
        root.findViewById<View>(R.id.miscBtnB).nextFocusLeftId = R.id.miscBtnA
    }

    /** A circular button background built in code — no new drawable files. */
    private fun ovalBackground(): GradientDrawable = GradientDrawable().apply {
        shape = GradientDrawable.OVAL
        setColor(ContextCompat.getColor(requireContext(), R.color.cardBg))
        setStroke(2, ContextCompat.getColor(requireContext(), R.color.focusBorder))
    }

    private fun toast(message: String) {
        Toast.makeText(requireContext(), message, Toast.LENGTH_SHORT).show()
    }
}

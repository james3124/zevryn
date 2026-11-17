package com.zevryn.app.screens.controller

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.Toast
import androidx.fragment.app.Fragment
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import com.zevryn.app.R

/**
 * "Gamepad" screen for the Zevryn launcher — a simplified, offline-safe take
 * on gamepad calibration (cf. GameHub's GamePadCalibrationActivity).
 *
 * Shows a focusable controller schematic (D-pad + A/B/X/Y), a Calibrate action
 * that only shows a toast, and a list of mapped buttons from local demo data.
 * No native/JNI calls are made, so the screen cannot crash on a missing core.
 */
class GamepadFragment : Fragment() {

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View = inflater.inflate(R.layout.ctrl_gamepad, container, false)

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        super.onViewCreated(view, savedInstanceState)

        // Each schematic button is focusable and reports a demo toast on click.
        SCHEMATIC_BUTTONS.forEach { (viewId, labelRes) ->
            view.findViewById<View>(viewId).setOnClickListener {
                val label = getString(labelRes)
                Toast.makeText(
                    requireContext(),
                    getString(R.string.ctrl_gamepad_button_toast, label),
                    Toast.LENGTH_SHORT
                ).show()
            }
        }

        view.findViewById<View>(R.id.ctrlCalibrateButton).setOnClickListener {
            Toast.makeText(
                requireContext(),
                R.string.ctrl_calibrate_toast,
                Toast.LENGTH_SHORT
            ).show()
        }

        val recycler = view.findViewById<RecyclerView>(R.id.ctrlGamepadBindings)
        recycler.layoutManager = LinearLayoutManager(requireContext())
        recycler.adapter = CtrlBindingAdapter(ButtonMappingFragment.demoBindings()) { binding ->
            Toast.makeText(
                requireContext(),
                getString(R.string.ctrl_gamepad_button_toast, getString(binding.actionRes)),
                Toast.LENGTH_SHORT
            ).show()
        }
    }

    private companion object {
        val SCHEMATIC_BUTTONS = listOf(
            R.id.ctrlDpadUp to R.string.ctrl_dpad_up,
            R.id.ctrlDpadDown to R.string.ctrl_dpad_down,
            R.id.ctrlDpadLeft to R.string.ctrl_dpad_left,
            R.id.ctrlDpadRight to R.string.ctrl_dpad_right,
            R.id.ctrlFaceA to R.string.ctrl_face_a,
            R.id.ctrlFaceB to R.string.ctrl_face_b,
            R.id.ctrlFaceX to R.string.ctrl_face_x,
            R.id.ctrlFaceY to R.string.ctrl_face_y
        )
    }
}

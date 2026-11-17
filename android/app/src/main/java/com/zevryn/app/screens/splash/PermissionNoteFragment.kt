package com.zevryn.app.screens.splash

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.Button
import android.widget.Toast
import androidx.fragment.app.Fragment
import com.zevryn.app.R

/**
 * Simple "permissions" information screen.
 *
 * The reference launcher gates on device permissions during startup. The
 * portable build performs NO real permission requests — the Rust core is
 * offline-capable — so this fragment only explains that fact. It can be shown
 * from the splash flow or hosted later by the parent session; no account,
 * cloud or telemetry is involved.
 */
class PermissionNoteFragment : Fragment() {

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View = inflater.inflate(R.layout.splash_permission, container, false)

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        super.onViewCreated(view, savedInstanceState)
        view.findViewById<Button>(R.id.permissionAck).setOnClickListener {
            Toast.makeText(
                requireContext(),
                getString(R.string.settings_hint),
                Toast.LENGTH_SHORT
            ).show()
        }
    }
}

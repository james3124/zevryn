package com.zevryn.app.screens.misc

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import androidx.fragment.app.Fragment
import com.zevryn.app.R

/**
 * "About" screen for the Zevryn launcher (grounded in GameHub Lite's
 * PreviewImageActivity / ProductDocActivity / menu screens).
 *
 * All content is static and offline: version, Rust-core note, a static module
 * count and a short note describing the reimplementation. No native calls.
 */
class AboutFragment : Fragment() {

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View = inflater.inflate(R.layout.misc_about, container, false)

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        super.onViewCreated(view, savedInstanceState)

        // Values live in resources (misc_*); binding them here keeps the
        // screen explicit and avoids any dependency on the Rust core.
        view.findViewById<TextView>(R.id.miscAboutVersion).text =
            getString(R.string.misc_about_version)
        view.findViewById<TextView>(R.id.miscAboutCore).text =
            getString(R.string.misc_about_core)
        view.findViewById<TextView>(R.id.miscAboutModules).text =
            getString(R.string.misc_about_modules)
        view.findViewById<TextView>(R.id.miscAboutNote).text =
            getString(R.string.misc_about_note)
    }
}

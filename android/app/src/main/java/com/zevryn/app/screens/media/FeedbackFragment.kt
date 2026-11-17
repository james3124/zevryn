package com.zevryn.app.screens.media

import android.content.Context
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.EditText
import android.widget.Toast
import androidx.appcompat.widget.AppCompatButton
import androidx.fragment.app.Fragment
import com.zevryn.app.R

/**
 * "Feedback" screen for the Zevryn launcher. Grounded in the reference
 * feedback screens, but reduced to a local-only form: the text is stored in
 * SharedPreferences and a toast is shown. No network, account or cloud.
 */
class FeedbackFragment : Fragment() {

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View = inflater.inflate(R.layout.media_feedback, container, false)

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        super.onViewCreated(view, savedInstanceState)

        val input = view.findViewById<EditText>(R.id.feedbackInput)
        val send = view.findViewById<AppCompatButton>(R.id.feedbackSend)

        send.setOnClickListener {
            val text = input.text.toString().trim()
            val prefs = requireContext().getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
            prefs.edit()
                .putString(KEY_LAST, text)
                .putInt(KEY_COUNT, prefs.getInt(KEY_COUNT, 0) + 1)
                .apply()
            input.text.clear()
            Toast.makeText(
                requireContext(),
                R.string.media_feedback_saved,
                Toast.LENGTH_SHORT
            ).show()
        }
    }

    companion object {
        private const val PREFS_NAME = "zevryn_media_feedback"
        private const val KEY_LAST = "last_feedback"
        private const val KEY_COUNT = "feedback_count"
    }
}

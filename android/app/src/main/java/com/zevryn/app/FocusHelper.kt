package com.zevryn.app

import android.view.View

/**
 * Wires D-pad focus neighbours for a RecyclerView laid out as a uniform grid.
 * The card views get stable, unique ids so Android's focus search can move
 * focus left/right/up/down with a gamepad.
 */
object FocusHelper {
    // Chosen outside the framework (0x0100...) and app (0x7f...) id ranges.
    private const val FOCUS_ID_BASE = 0x20000000

    fun wireGridFocus(view: View, position: Int, count: Int, columns: Int) {
        if (columns <= 0 || count <= 0) return
        view.id = FOCUS_ID_BASE + position

        val rows = (count + columns - 1) / columns
        val row = position / columns
        val col = position % columns

        fun idAt(r: Int, c: Int): Int {
            if (r < 0 || c < 0 || r >= rows || c >= columns) return View.NO_ID
            val index = r * columns + c
            return if (index < count) FOCUS_ID_BASE + index else View.NO_ID
        }

        view.nextFocusLeftId = idAt(row, col - 1)
        view.nextFocusRightId = idAt(row, col + 1)
        view.nextFocusUpId = idAt(row - 1, col)
        view.nextFocusDownId = idAt(row + 1, col)
    }
}

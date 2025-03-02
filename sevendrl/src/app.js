import init, { init_game } from '../pkg/sevendrl.js'

document.addEventListener('DOMContentLoaded', () => {
    init().then(async () => {
        const wrapper = await init_game('main_canvas', Math.random())

        const handleEvent = (e) => {
            e.preventDefault()
            const rect = e.target.getBoundingClientRect();
            const x = e.clientX - rect.left;
            const y = e.clientY - rect.top;
            if (e.type === 'mousedown') { e.target.focus() } // If we click it, focus it in addition to whatever else
            wrapper.mouse_event(e.type, x, y)
        }

        const handleKey = (e) => {
            // Ignore any strokes that involve ctrl or alt, so we don't eat keys people
            // might want, like C-r
            if (!e.ctrlKey && !e.altKey && !e.metaKey) {
                e.preventDefault()
                wrapper.key(e.key)
            }
        }

        const canvas = document.getElementById('main_canvas')
        canvas.addEventListener('mousedown', handleEvent)
        canvas.addEventListener('mouseup', handleEvent)
        canvas.addEventListener('mousemove', handleEvent)

        canvas.addEventListener('keydown', handleKey)

        canvas.focus()

        let time = 0
        const redraw = (newTime) => {
            wrapper.redraw(newTime - time)
            time = newTime
            requestAnimationFrame(redraw)
        }

        redraw(8)
    })
})
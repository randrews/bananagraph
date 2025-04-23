import init, { init_game } from '../pkg/rpg.js'

document.addEventListener('DOMContentLoaded', () => {
    init().then(async () => {
        const canvas = document.getElementById('main_canvas')

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

        canvas.addEventListener('mousedown', handleEvent)
        canvas.addEventListener('mouseup', handleEvent)
        canvas.addEventListener('mousemove', handleEvent)

        canvas.addEventListener('keydown', handleKey)

        canvas.focus()

        const resize = () => {
            const step = 16
            const maxHeight = Math.floor(window.innerHeight / step) * step
            const maxWidth = Math.floor(window.innerWidth / step) * step
            const widthFromMaxHeight = Math.floor(maxHeight * 4 / 3 / step) * step
            const heightFromMaxWidth = Math.floor( maxWidth * 3 / 4 / step) * step

            if (window.innerWidth < 800 || window.innerHeight < 600) {
                canvas.width = 800
                canvas.height = 600
            } else if (widthFromMaxHeight < window.innerWidth) {
                canvas.width = widthFromMaxHeight
                canvas.height = maxHeight
            } else {
                canvas.width = maxWidth
                canvas.height = heightFromMaxWidth
            }
            console.log([canvas.width, canvas.width % step,
                canvas.height, canvas.height % step])
            wrapper.resize(canvas.width, canvas.height)
        }
        addEventListener('resize', resize)
        resize()

        let time = 0
        const redraw = (newTime) => {
            wrapper.redraw(newTime - time)
            time = newTime
            requestAnimationFrame(redraw)
        }

        redraw(8)
    })
})
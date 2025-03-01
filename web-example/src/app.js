import init, { init_gpu_wrapper, draw_toast } from '../pkg/web_example.js'

document.addEventListener('DOMContentLoaded', () => {
    init().then(async () => {
        const wrapper = await init_gpu_wrapper('main_canvas')
        draw_toast(wrapper)
        // const canvas = document.querySelector('canvas')
        // const game = new Game();
        // const display = new Display();
        // const ctx = canvas.getContext('2d')
        //
        // const handleEvent = (e) => {
        //     e.preventDefault()
        //     const rect = e.target.getBoundingClientRect();
        //     const x = e.clientX - rect.left;
        //     const y = e.clientY - rect.top;
        //     display.event(e.type, x, y)
        // }
        //
        // const handleKey = (e) => {
        //     const importantKeys = [
        //         'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight',
        //         'a', 'b', 'c', 'd', 'e', 'f', 's', ' ', 'Enter', 'Escape'
        //     ]
        //
        //     if (importantKeys.indexOf(e.key) !== -1) {
        //         e.preventDefault()
        //         display.key(e.key)
        //     }
        // }
        //
        // canvas.addEventListener('mousedown', handleEvent)
        // canvas.addEventListener('mouseup', handleEvent)
        // canvas.addEventListener('mousemove', handleEvent)
        // canvas.addEventListener('keydown', handleKey)
        //
        // canvas.focus()
        // const redraw = (_time) => {
        //     display.draw(game, ctx)
        //     requestAnimationFrame(redraw)
        // }
        //
        // redraw(0)
    })
})
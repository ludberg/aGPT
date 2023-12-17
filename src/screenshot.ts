import { appWindow } from '@tauri-apps/api/window';
import './style.css'
import { invoke } from "@tauri-apps/api";
import { emit } from '@tauri-apps/api/event'

async function closeWindow() {
    await appWindow.close();
}

document.addEventListener('keydown', function (event) {
    console.log(event.keyCode);
    if (event.keyCode == 27) { // esc

        closeWindow();
        emit('showmainwindow', {
            theMessage: 'triggered from On ESC pressed in screenshot.ts',
        });
    }
});

document.addEventListener("DOMContentLoaded", () => {
    emit('hidemainwindow', {
        theMessage: 'triggered from On Document Ready in screenshot.ts',
    });
});


initDraw(<HTMLDivElement>document.getElementById('canvas'));

function initDraw(canvas: HTMLDivElement) {
    function setMousePosition(e: MouseEvent) {
        var ev = e || window.event; //Moz || IE
        if (ev.pageX) { //Moz
            mouse.x = ev.pageX + window.pageXOffset;
            mouse.y = ev.pageY + window.pageYOffset;
        } else if (ev.clientX) { //IE
            mouse.x = ev.clientX + document.body.scrollLeft;
            mouse.y = ev.clientY + document.body.scrollTop;
        }
    };

    var mouse = {
        x: 0,
        y: 0,
        startX: 0,
        startY: 0
    };
    var element: HTMLDivElement | null = null;

    canvas.onmousemove = function (e: MouseEvent) {
        setMousePosition(e);
        if (element !== null) {
            element.style.width = Math.abs(mouse.x - mouse.startX) + 'px';
            element.style.height = Math.abs(mouse.y - mouse.startY) + 'px';
            element.style.left = (mouse.x - mouse.startX < 0) ? mouse.x + 'px' : mouse.startX + 'px';
            element.style.top = (mouse.y - mouse.startY < 0) ? mouse.y + 'px' : mouse.startY + 'px';
        }
    }

    // canvas.onmouseup = function () {
    //     element = null;
    //     canvas.style.cursor = "default";
    //     console.log(mouse);
    //     invoke('take_screenshot', { startX: mouse.startX, startY: mouse.startY, endX: mouse.x, endY: mouse.y }).then(function (response) {
    //         console.log(response);
    //         closeWindow();
    //     });
    // }

    //canvas.onmousedown = function () {
    canvas.onclick = function () {
        if (element !== null) {
            element = null;
            canvas.style.cursor = "default";
            console.log(mouse);
            invoke('take_screenshot', { startX: mouse.startX, startY: mouse.startY, endX: mouse.x, endY: mouse.y }).then(function (response) {
                console.log(response);
                closeWindow();
            });
        } else {
            canvas.style.background = "rgba(0, 0, 0, 0.0)";
            mouse.startX = mouse.x;
            mouse.startY = mouse.y;
            element = document.createElement('div');
            element.className = 'rectangle'
            element.style.left = mouse.x + 'px';
            element.style.top = mouse.y + 'px';
            canvas.appendChild(element)
            canvas.style.cursor = "crosshair";
        }
    }
}
import { appWindow, LogicalSize, PhysicalPosition } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api";
//import { listen, emit } from "@tauri-apps/api/event";
import { pictureDir, join, appDataDir } from '@tauri-apps/api/path';
import { convertFileSrc } from '@tauri-apps/api/tauri';
import * as showdown from "showdown";
//import { Store } from "tauri-plugin-store-api";
//const store = new Store(".settings.dat");

import spinner from './img/spinner2.gif';
import { Store } from "tauri-plugin-store-api";
const store = new Store(".settings.dat");


export function streamResponse(response_bit: string) {
    const div_response = document.getElementById("div_response");

    if (div_response != null) {
        div_response.innerHTML = div_response.innerHTML + response_bit;
    }
}


export async function getPictureDir() {
    return await pictureDir();
}

export async function getAppDataDir() {
    return await appDataDir();
}


export async function makeFullPath(pictureDirPath: string, filename: string) {
    //console.log("makeFullPath: " + pictureDirPath + "gpt/" + filename);
    return await join(pictureDirPath, "gpt/" + filename);

}


export async function getPictureAsset(filename: string) {
    return await getPictureDir().then(function (pictureDir) {
        return makeFullPath(pictureDir, filename).then(function (fullPath) {
            //console.log("fullpath: " + fullPath);
            const assetUrl = convertFileSrc(fullPath);
            //console.log("assetURL: " + assetUrl);

            return assetUrl;

        });
    });
}

export function reset_img_screenshot() {
    const img_screenshot = <HTMLImageElement>document.getElementById('img_screenshot');
    const screenshot_upload_wrapper = <HTMLDivElement>document.getElementById('screenshot_upload_wrapper');
    if (img_screenshot !== null && screenshot_upload_wrapper !== null) {
        // img_screenshot.src = "";
        // screenshot_upload_wrapper.style.display = "none";
        screenshot_upload_wrapper.remove();
        return true;
    }
    return false;
}
export async function save_key_to_store(key: string, keyValue: string) {
    await store.set(key, keyValue);

    // var val = await store.get("api-key");
    await store.save();
}

export async function get_key_from_store(key: string) {

    var val = await store.get(key);
    if (val) {
        return val;
    }
    return "";
}

export function setNormalWindowSizeAndPosition() {
    // Hämta nuvarande storlek på fönstret
    appWindow.innerSize().then((innerSize) => {

        // Kollar om fönstret har default-storleken det får när appen öppnar
        // Om det har andra dimensioner har användaren ändrat storlek, eller så har
        // storleken på fönstret ändrats som en följd av att man skickat något till ChatGPT,
        // eftersom fönstret då blir större för att visa prompt och svar. 
        //console.log("innserSize: " + innerSize.height + "/" + innerSize.width);
        if (innerSize.height == 100 && innerSize.width == 800) {


            // Ändrar från 800x100 till 800x600 
            appWindow.setSize(new LogicalSize(800, 600));


            // Påbörjar centrering av appen som nu fått ny storlek
            // Hämtar nuvarande monitor där appen finns, och dess size
            appWindow.innerPosition().then((innerPosition) => {
                if (innerPosition !== null) {
                    //console.log("Inner position: " + innerPosition.x + "/" + innerPosition.y);
                    appWindow.setPosition(new PhysicalPosition(innerPosition.x, innerPosition.y - 300));
                }
            })
        }
    });
}


export function textAreaAdjust(element: HTMLElement) {
    //console.log("textAreaAdjust() called");
    element.style.height = "5px";
    element.style.height = (element.scrollHeight - 55) + "px";
}

export function toggleSettings() {

    const settings_div = document.getElementById('settings');
    const content_div = document.getElementById('content');

    if (settings_div !== null && content_div !== null) {
        //console.log("toggleSettings() called: " + window.getComputedStyle(settings_div, null).display);
        if (settings_div.style.display == "none") {
            console.log("Settings är none");
            settings_div.style.display = "block";
            content_div.style.display = "none";
        } else {
            console.log("Settings är block");
            settings_div.style.display = "none";
            content_div.style.display = "block";
        }
    }
}


export function handle(e: KeyboardEvent) {



    if (e.keyCode === 13 && !e.shiftKey) {
        e.preventDefault();

        // Kollar så vi har en API-nyckel
        var hasApiKey = false;
        const setting_api_key = document.getElementById('setting_api_key');
        if (setting_api_key !== null) {
            if ((<HTMLInputElement>setting_api_key).value.length > 0) {
                hasApiKey = true;
            }
        }

        if (!hasApiKey) {
            // Vi har ingen API-nyckel. Visa settings-fönstret
            const settings_div = document.getElementById('settings');

            const content_div = document.getElementById('content');
            if (content_div !== null && settings_div !== null) {
                content_div.style.display = "none";
                settings_div.style.display = "block";
            }
        } else {
            // Dölj settings-fönstret om det är synligt
            const settings_div = document.getElementById('settings');

            const content_div = document.getElementById('content');
            if (content_div !== null && settings_div !== null) {
                content_div.style.display = "block";
                settings_div.style.display = "none";
            }

            var textarea = <HTMLInputElement>document.getElementById('prompt_input');

            if (textarea !== null) {

                var prompt_input = textarea.value.trim();

                textarea.value = '';

                reset_img_screenshot();


                if (prompt_input.length) {




                    createPrompt(prompt_input);
                    createResponse();
                    setNormalWindowSizeAndPosition();

                    invoke('call_chatgpt', { prompt: prompt_input }).then(function (_response) {
                        //console.log("Response: " + response);

                    });

                }
            }
        }
    }
}

export function scrollToBottom() {
    const content_div = document.getElementById('content');
    if (content_div !== null) {
        // Skrollar ner content-diven så den nya prompten syns
        content_div.scrollTop = content_div.scrollHeight;
    }
}

export function createPrompt(prompt_input: string) {
    const content_div = document.getElementById('content');

    // Skapar en div som håller prompten
    const div_prompt_wrapper = document.createElement('div');
    div_prompt_wrapper.classList.add('prompt_wrapper');


    get_key_from_store("latest-screenshot").then(function (filename) {
        if (String(filename).length > 0) {

            getPictureAsset(String(filename)).then(function (assetUrl) {
                const div_empty = document.createElement('div');
                const img_screenshot = document.createElement('img');
                img_screenshot.classList.add('prompt_screenshot');
                img_screenshot.src = assetUrl;
                div_empty.appendChild(img_screenshot);

                div_prompt_wrapper.appendChild(div_empty);


            }).catch(function (e) {
                console.log("Error: getPictureAsset(): " + e);
            });
        }

    });

    const div_prompt = document.createElement('div');
    div_prompt.innerHTML = prompt_input;
    div_prompt.classList.add('prompt');
    div_prompt_wrapper.appendChild(div_prompt);

    if (content_div !== null) {

        // Lägger till den nya prompten.
        content_div.appendChild(div_prompt_wrapper);

        scrollToBottom();
    }

}

export function createResponse() {

    const content_div = document.getElementById('content');


    const div_response_wrapper = document.createElement('div');
    div_response_wrapper.classList.add('response_wrapper');
    const span_bot_name = document.createElement('span');
    span_bot_name.innerHTML = 'ChatGPT';
    span_bot_name.classList.add('bot_name');
    div_response_wrapper.appendChild(span_bot_name);

    const div_response = document.createElement('div');
    div_response.setAttribute("id", "div_response");
    div_response.classList.add('response');

    const img_loader = document.createElement('img');
    img_loader.setAttribute("id", "img_loader");
    img_loader.src = spinner;
    img_loader.classList.add('loading_icon');
    div_response.appendChild(img_loader);

    //div_response.innerHTML = 'test debug';
    div_response_wrapper.appendChild(div_response);

    if (content_div !== null) {

        // Lägger till den nya prompten.
        content_div.appendChild(div_response_wrapper);
        scrollToBottom();
    }
}

export function removeSpinner() {
    // Kollar om vi har ett tillfälligt response, dvs gif:en "spinner" som visar att vi väntar på ChatGPT
    const img_loader = document.getElementById("img_loader");
    if (img_loader != null) {
        img_loader.remove();
    }
}

export function formatResponseAsMarkdown() {
    // Uppdaterar response-rutan med rätt text
    const div_response = document.getElementById("div_response");

    if (div_response != null) {

        var raw_text = div_response.innerHTML;

        // Gör om svaret till markdown
        var converter = new showdown.Converter(),
            //var converter = showdown.Converter(),
            text = String(raw_text),
            html = converter.makeHtml(text);

        div_response.innerHTML = html;

    }
}

export function updateResponse(response: string) {
    const content_div = document.getElementById('content');

    removeSpinner();

    // Uppdaterar response-rutan med rätt text
    const div_response = document.getElementById("div_response");

    if (div_response != null && content_div !== null) {
        // Gör om svaret till markdown
        //var showdown  = require('showdown'),
        var converter = new showdown.Converter(),
            //var converter = showdown.Converter(),
            text = String(response),
            html = converter.makeHtml(text);

        div_response.innerHTML = html;
        div_response.removeAttribute("id");

        scrollToBottom();
    }
}


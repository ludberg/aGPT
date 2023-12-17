import { appWindow } from '@tauri-apps/api/window';
import { invoke } from "@tauri-apps/api";
import { listen, emit } from "@tauri-apps/api/event";
import { register } from '@tauri-apps/api/globalShortcut';
import { textAreaAdjust, toggleSettings, handle, reset_img_screenshot, save_key_to_store, getPictureAsset, formatResponseAsMarkdown, scrollToBottom } from './gpt.ts'
import { setNormalWindowSizeAndPosition, createPrompt, createResponse, updateResponse, streamResponse, removeSpinner, getPictureDir, getAppDataDir } from './gpt.ts'
import { open } from '@tauri-apps/api/shell';

import './style.css'
import img_settings from './img/Settings-PNG-Picture.png'
import img_camera from './img/camera_icon.png'
import img_close from './img/close_icon.png'








document.addEventListener("DOMContentLoaded", () => {
  const settings_div = document.getElementById('settings');

  if (settings_div !== null) {
    settings_div.style.display = "none";
  }


});




document.querySelector<HTMLDivElement>('#app')!.innerHTML = `
<div class="container">
<div id="settings" style="display: none;">
  <h3>Settings</h3>
  API key: <input type="text" id="setting_api_key" value="" /><br>
  Model: <select id="setting_model">
    <option value="gpt-4-1106-preview">GPT 4</option>
    <option value="gpt-3.5-turbo">GPT 3.5</option>
  </select><br>
  <div class="error" id="settings_error"></div>
  <div>
    <span class="link" id="link_log_file">Open log folder</span><br />
    <span class="link" id="link_picture_folder">Open screenshot folder</span>
  </div>
</div>
<div id="content" style="display: block;">

  
 
  
  
</div>


<div id="footer">
  <div id="prompt_input_wrapper">
 
    <textarea id="prompt_input" , rows="1"
      autofocus></textarea>
      <div id="icon_wrapper" >
      <img id="camera_icon" src="`+ img_camera + `" title="Snippet part of the screen (Ctrl+P)"
        class="icon" />
    <img id="settings_icon" src="`+ img_settings + `" title="Settings"
    class="icon"  />
  </div>  
  </div>
  


</div>
</div>
`



async function showWindow() {
  await appWindow.show();
}

async function closeWindow() {
  await appWindow.close();
}

async function openPictureDir(directory: string) {
  await open(directory);
}

const link_picture_folder = document.getElementById('link_picture_folder');
if (link_picture_folder !== null) {
  link_picture_folder.addEventListener('click', function () {
    getPictureDir().then(function (pictureDir) {
      openPictureDir(pictureDir + "gpt");
    });

  });
}

const link_log_file = document.getElementById('link_log_file');
if (link_log_file !== null) {
  link_log_file.addEventListener('click', function () {
    getAppDataDir().then(function (appDataDir) {
      openPictureDir(appDataDir);
    });

  });
}


const prompt_input = document.getElementById('prompt_input');
if (prompt_input !== null) {
  prompt_input.addEventListener('input', function () {
    textAreaAdjust(this);
  });

  prompt_input.addEventListener('keydown', function (event) {
    handle(event);
  })
}

const settings_icon = document.getElementById('settings_icon');
if (settings_icon !== null) {
  settings_icon.addEventListener('click', function () {
    setNormalWindowSizeAndPosition();
    toggleSettings();

  });
}

const camera_icon = document.getElementById('camera_icon');
if (camera_icon !== null) {
  camera_icon.addEventListener('click', function () {
    //toggleSettings();
    setNormalWindowSizeAndPosition();
    invoke('show_screenshot_window');
  });
}



const setting_api_key = document.getElementById('setting_api_key');
if (setting_api_key !== null) {
  setting_api_key.addEventListener('change', function () {

    const settings_error = document.getElementById('settings_error');


    var new_api_key = (<HTMLInputElement>this).value;

    invoke('save_to_store', { key: "api-key", keyValue: new_api_key }).then(function (response) {
      console.log(response);
      if (response == "success") {
        if (new_api_key.length == 0) {
          if (settings_error !== null) {
            settings_error.style.display = "block";
            settings_error.innerHTML = "Missing API key";
          }
        } else {
          if (settings_error !== null) {
            settings_error.style.display = "none";
          }
        }
      } else {
        if (settings_error !== null) {
          settings_error.innerHTML = "Unable to save api key";
          settings_error.style.display = "none";
        }
      }
    });
  });
}

invoke('get_key_from_store', { key: "api-key" }).then((api_key) => {
  if (String(api_key).length > 0) {
    const settings_api_key = <HTMLInputElement>document.getElementById('setting_api_key');
    if (setting_api_key !== null) {
      settings_api_key.value = String(api_key);
    }
  } else {
    // Flyttar fokus till settings vyn och skriver att vi saknar en key
    const settings_error = document.getElementById('settings_error');
    if (settings_error !== null) {
      settings_error.style.display = "block";
      settings_error.innerHTML = "Missing API key";
    }

    const settings_div = document.getElementById('settings');
    const content_div = document.getElementById('content');

    if (content_div !== null && settings_div !== null) {
      content_div.style.display = "none";
      settings_div.style.display = "block";
    }
  }
});

invoke('send_prompt_to_frontend', { window: appWindow })
  .then((prompt) => {
    //console.log("Prompt from backend: " + prompt);
    if (String(prompt).length > 0) {
      createPrompt(String(prompt));
      createResponse();
      setNormalWindowSizeAndPosition();
      scrollToBottom();
    }
  })
  .catch(err => {
    console.error("Finns inget prompt: " + err);
    return err;
  });

invoke('send_response_to_frontend', { window: appWindow })
  .then(response => {
    //console.log("Response from backend: " + response);

    updateResponse(String(response));
  })
  .catch(err => {
    //console.error("Finns inget response: " + err);
    return err;
  })



// listen("show_prompt_in_frontend", ev => {

// });

listen("hide_spinner_in_frontend", () => {
  removeSpinner();
});

listen("format_response_in_frontend", () => {
  formatResponseAsMarkdown();
});



listen("stream_response_in_frontend", ev => {
  var x = ev.payload;
  if (x && typeof x === "object") {
    if ("data" in x) {
      //console.log(x.data);
      streamResponse(String(x.data));
      scrollToBottom();
    }
  }
});


listen("show_response_in_frontend", ev => {
  //console.log("fick show_response_in_frontend:");
  var x = ev.payload;
  if (x && typeof x === "object") {
    if ("data" in x) {
      updateResponse(String(x.data));
    }
  }

});




listen("show-latest-screenshot", ev => {
  showWindow();
  reset_img_screenshot();
  setNormalWindowSizeAndPosition();

  var x = ev.payload;
  if (x && typeof x === "object") {
    if ("data" in x) {
      //console.log(x.message);
      var filename = String(x.data);

      getPictureAsset(filename).then(function (assetUrl) {

        //const img_screenshot = <HTMLImageElement>document.getElementById('img_screenshot');
        //const screenshot_upload_wrapper = <HTMLDivElement>document.getElementById('screenshot_upload_wrapper');


        // if (img_screenshot !== null && screenshot_upload_wrapper !== null) {
        //   img_screenshot.src = assetUrl;
        //   screenshot_upload_wrapper.style.display = "block";

        const content_div = document.getElementById('content');
        if (content_div !== null) {


          const screenshot_upload_wrapper = document.createElement('div');
          screenshot_upload_wrapper.setAttribute("id", "screenshot_upload_wrapper");
          const img_screenshot_close = document.createElement('img');
          img_screenshot_close.src = img_close;
          img_screenshot_close.setAttribute("id", "img_screenshot_close");



          screenshot_upload_wrapper.appendChild(img_screenshot_close);

          const img_screenshot = document.createElement('img');
          img_screenshot.src = assetUrl;
          img_screenshot.setAttribute("id", "img_screenshot");

          screenshot_upload_wrapper.append(img_screenshot);

          //footer_div.insertBefore(screenshot_upload_wrapper, footer_div.firstChild);
          content_div.appendChild(screenshot_upload_wrapper);

          //const content_div = document.getElementById('content');
          if (content_div !== null) {

            content_div.scrollTop = content_div.scrollHeight;
          }

          if (img_screenshot_close !== null) {
            img_screenshot_close.addEventListener('click', function () {
              reset_img_screenshot();
              save_key_to_store("latest-screenshot", "");
            });
          }

        }



      }).catch(function (e) {
        console.log("Error: getPictureAsset(): " + e);
      });




    }
  }


});

async function getVisibility() {
  return await appWindow.isVisible().
    then(visibility => {
      return visibility;
    });
}

async function registerKeyboardShortcuts() {

  await register('Esc', () => {

    getVisibility()
      .then(visibility => {
        if (visibility) {

          if (reset_img_screenshot() == true) {
            //   reset_img_screenshot() returnerar true om det finns ett screenshot att ta bort, annars false
          } else {
            closeWindow();
          }

        } else {
          emit('close_screenshot_window');
          emit('showmainwindow', {
            theMessage: 'triggered from On ESC pressed in screenshot.ts',
          });
        }
      });

  });

  await register('CommandOrControl+P', () => {
    invoke('show_screenshot_window');
  });

}

registerKeyboardShortcuts();
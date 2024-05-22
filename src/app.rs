use leptos::leptos_dom::ev::SubmitEvent;
use leptos::*;
use serde::{Deserialize, Serialize};
use tauri_sys::tauri;

// #[wasm_bindgen]
// extern "C" {
//     #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
//     async fn invoke(cmd: &str, args: JsValue) -> JsValue;
//     #[wasm_bindgen(js_namespace = ["console"])]
//     async fn log(args: JsValue) -> JsValue;
// }

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum AudioPath {
    Link(String),
    Path(String),
}

#[derive(Serialize, Deserialize)]
struct PlayArgs {
    files: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct LoadFileArgs {
    link: String,
}

#[component]
pub fn App() -> impl IntoView {
    // let (file_path, set_file_path) = create_signal(String::new());
    let (err_msg, set_err_msg) = create_signal(None);
    let (link, set_link) = create_signal(String::new());
    let (audio_list, set_audio_list) = create_signal(Vec::new());

    let update_link = move |ev| {
        let v = event_target_value(&ev);
        set_link.set(v);
    };

    // let select_file = move |ev: leptos::ev::Event| {
        //let v = event_target(&ev);
        //set_file_path.set(v);

        //if ().is_some() {}

        // if v.target.files[0] {
        //     document.body.append('You selected ' + e.target.files[0].name);
        //   }
    // };

    let load_file = move |ev: SubmitEvent| {
        ev.prevent_default();
        spawn_local(async move {
            let link = link.get_untracked();

            if link.is_empty() {
                return;
            }
            set_link.set(String::new());
            let mut new_list = audio_list.get_untracked();
            new_list.push(link.clone());
            set_audio_list.set(new_list);
            let res: Result<(), tauri_sys::Error> = tauri::invoke("load_file", &LoadFileArgs { link }).await;
            if let Err(err) = res {
                set_err_msg.set(Some(err.to_string()));
            }
            set_link.set(String::new());
            // let audio_list = audio_list.get_untracked();
            // let res: Result<(), tauri_sys::Error> = tauri::invoke("play", &PlayArgs { files: audio_list }).await;
            // if let Err(err) = res {
            //     set_err_msg.set(Some(err.to_string()));
            // }
        });
    };

    let play_audios = move |ev: SubmitEvent| {
        ev.prevent_default();
        spawn_local(async move {
            let audio_list = audio_list.get_untracked();
            let res: Result<(), tauri_sys::Error> = tauri::invoke("play", &PlayArgs { files: audio_list }).await;
            if let Err(err) = res {
                set_err_msg.set(Some(err.to_string()));
            }
        });
    };

    view! {
        <main class="container">
        <Show
        when=move || { err_msg.get().is_some() }
        fallback=|| view! {
            
         }
      >
      <p>{err_msg.get().unwrap_or(String::new())}</p>
      </Show>
            <div class="column" align="left">
            <Show
            when=move || { audio_list.get().len() > 0 }
            fallback=|| view! {
                <p>"Add audio links to play!"</p>
             }
          >
          <div>
          <ul>
          {audio_list.get().into_iter()
              .map(|n| view! { <li>{n}</li>})
              .collect::<Vec<_>>()}
            </ul>
            </div>
            <div>
          <form class="row" on:submit=play_audios>

          <button type="submit">"Play"</button>

            </form>
            </div>
          </Show>

            </div>

            <form class="row" on:submit=load_file>
            <input
                id="link-input"
                placeholder="Enter a link..."
                on:input=update_link
                prop:value=link
            />

            // <input
            //     id="file-input"
            //     placeholder="Select file..."
            //     on:input=select_file
            //     //prop:value=file_path
            // />

            <button type="submit">"Add"</button>
        </form>

        </main>
    }
}

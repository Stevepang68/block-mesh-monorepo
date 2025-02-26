use crate::components::logo::Logo;
use crate::utils::extension_wrapper_state::ExtensionWrapperState;
use block_mesh_common::constants::BLOCKMESH_VERSION;
use leptos::*;
use leptos_use::{use_clipboard, UseClipboardReturn};

#[component]
pub fn ExtensionLoggedIn() -> impl IntoView {
    let UseClipboardReturn { copy, .. } = use_clipboard();
    let state = use_context::<ExtensionWrapperState>().unwrap();
    let copy_to_clipboard = move |_| {
        let invite_code_string = state.invite_code.get_untracked();
        if !invite_code_string.is_empty() {
            copy(&format!(
                "https://app.blockmesh.xyz/register?invite_code={}",
                invite_code_string
            ));
            state.set_success("Successfully Copied");
        } else {
            state.set_error("Failed to copy invite code");
        }
    };

    let logout = create_action(move |_| async move {
        state.clear().await;
    });

    view! {
        <div class="auth-card">
            <img
                class="background-image"
                src="https://r2-images.blockmesh.xyz/dc54851e-a585-44af-e6b6-18d16c984500.png"
                alt="background"
            />
            // <div class="auth-card-frame"></div>
            <div class="auth-card-top">
                <div class="auth-card-logout">
                    <svg
                        title="logout"
                        xmlns="http://www.w3.org/2000/svg"
                        height="24px"
                        viewBox="0 -960 960 960"
                        width="24px"
                        fill="#FF7E07"
                        on:click=move |_| {
                            logout.dispatch(());
                        }
                    >

                        <path d="M200-120q-33 0-56.5-23.5T120-200v-560q0-33 23.5-56.5T200-840h280v80H200v560h280v80H200Zm440-160-55-58 102-102H360v-80h327L585-622l55-58 200 200-200 200Z"></path>
                    </svg>
                </div>
            </div>
            <div class="auth-card-body">
                <Logo/>
                <div class="auth-card-content">
                    <div class="pulse"></div>
                    <small class="relative text-off-white">Version: {{ BLOCKMESH_VERSION }}</small>
                    <div class="auth-card-chip auth-card-user">
                        <strong>{move || state.email.get()}</strong>
                    </div>
                </div>
            </div>
            <div class="auth-card-bottom logged-bottom">
                <button class="auth-card-button font-bebas-neue text-off-white">
                    <a
                        href=move || {
                            let url = state.blockmesh_url.get();
                            format!("{}/ui/dashboard", url)
                        }

                        target="blank"
                    >
                        Dashboard
                    </a>
                </button>
                <button
                    class="auth-card-button font-bebas-neue text-off-white"
                    on:click=copy_to_clipboard
                >
                    Refer
                </button>
            </div>
        </div>
    }
}

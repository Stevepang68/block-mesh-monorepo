use crate::frontends::context::extension_state::ExtensionContext;
use crate::frontends::context::notification_context::NotificationContext;
use crate::frontends::frontend_extension::components::logo::Logo;
use crate::frontends::utils::auth::login;
use crate::frontends::utils::connectors::send_message_channel;
use block_mesh_common::chrome_storage::{AuthStatus, MessageKey, MessageType, MessageValue};
use block_mesh_common::interfaces::server_api::LoginForm;
use leptos::logging::log;
use leptos::*;
use uuid::Uuid;

#[component]
pub fn ExtensionLogin() -> impl IntoView {
    let state = use_context::<ExtensionContext>().unwrap();
    let notifications = expect_context::<NotificationContext>();
    let (password, set_password) = create_signal(String::new());
    let (email, set_email) = create_signal(String::new());
    let (wait, set_wait) = create_signal(false);

    let submit_action_resource = create_local_resource(
        move || (),
        move |_| async move {
            if wait.get_untracked()
                || email.get_untracked().is_empty()
                || password.get_untracked().is_empty()
            {
                return;
            }
            set_wait.set(true);
            let credentials = LoginForm {
                email: email.get_untracked(),
                password: password.get_untracked(),
            };

            let result = login(&state.blockmesh_url.get_untracked(), &credentials).await;
            match result {
                Ok(res) => {
                    if res.message.is_some() {
                        notifications.set_error(res.message.unwrap());
                        set_wait.set(false);
                        return;
                    }
                    if let Some(api_token) = res.api_token {
                        if api_token != state.api_token.get_untracked()
                            || state.api_token.get_untracked() == Uuid::default()
                        {
                            log!("Store new api token");
                            state.api_token.update(|v| *v = api_token);
                            state.email.update(|e| *e = credentials.email.clone());
                            send_message_channel(
                                MessageType::SET,
                                MessageKey::Email,
                                Option::from(MessageValue::String(state.email.get_untracked())),
                            )
                            .await;
                            send_message_channel(
                                MessageType::SET,
                                MessageKey::ApiToken,
                                Option::from(MessageValue::UUID(api_token)),
                            )
                            .await;
                        }
                        notifications.set_success("Successfully logged in");
                        state.status.update(|v| *v = AuthStatus::LoggedIn);
                    }
                }
                Err(e) => {
                    notifications.set_error(format!(
                        "Failed to login, please check your credentials again : {:?}",
                        e
                    ));
                }
            }
            set_wait.set(false);
        },
    );

    view! {
        <div class="auth-card">
            <img
                class="background-image"
                src="https://imagedelivery.net/3RKw_J_fJQ_4KpJP3_YgXA/2f6630f8-f48a-47ed-753b-4445c9399e00/public"
                alt="background"
            />
            <div class="auth-card-frame"></div>
            <div class="auth-card-top"></div>
            <div class="auth-card-body">
                <Logo/>
                <form on:submit=|ev| ev.prevent_default()>
                    <div class="auth-card-input-container">
                        <input
                            type="text"
                            required=""
                            name="email"

                            on:keyup=move |ev: ev::KeyboardEvent| {
                                let val = event_target_value(&ev);
                                set_email.update(|v| *v = val.to_ascii_lowercase());
                            }

                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                set_email.update(|v| *v = val.to_ascii_lowercase());
                            }
                        />

                        <label class="font-bebas-neue text-off-white">Email</label>
                    </div>
                    <div class="auth-card-input-container">
                        <input
                            type="password"
                            required=""

                            name="password"
                            on:keyup=move |ev: ev::KeyboardEvent| {
                                match &*ev.key() {
                                    "Enter" => {
                                        submit_action_resource.refetch();
                                    }
                                    _ => {
                                        let val = event_target_value(&ev);
                                        set_password.update(|p| *p = val);
                                    }
                                }
                            }

                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                set_password.update(|p| *p = val);
                            }
                        />

                        <label class="font-bebas-neue text-off-white">Password</label>
                    </div>
                    <br/>
                    <button
                        class="auth-card-button font-bebas-neue text-off-white"
                        on:click=move |_ev| {
                            submit_action_resource.refetch();
                        }
                    >

                        Login
                    </button>
                </form>
            </div>
            <div class="auth-card-bottom">
                <small class="font-open-sans text-orange">Doesnt have an account yet?</small>
                <br/>
                <button on:click=move |_| { state.status.update(|v| *v = AuthStatus::Registering) }>
                    <small
                        class="text-magenta underline cursor-pointer"
                        on:click=move |_| { state.status.update(|v| *v = AuthStatus::Registering) }
                    >
                        Register now
                    </small>
                </button>
            </div>
        </div>
    }
}

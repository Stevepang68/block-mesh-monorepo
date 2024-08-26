use crate::frontends::context::extension_state::ExtensionContext;
use crate::frontends::context::notification_context::NotificationContext;
use crate::frontends::frontend_extension::components::logo::Logo;
use crate::frontends::utils::auth::register;
use crate::frontends::utils::connectors::send_message_channel;
use block_mesh_common::chrome_storage::{AuthStatus, MessageKey, MessageType, MessageValue};
use block_mesh_common::interfaces::server_api::RegisterForm;
use leptos::*;
use leptos_dom::tracing;

#[component]
pub fn ExtensionRegister() -> impl IntoView {
    let state = use_context::<ExtensionContext>().unwrap();
    let notifications = expect_context::<NotificationContext>();
    let (password, set_password) = create_signal(String::new());
    let (email, set_email) = create_signal(String::new());
    let (invite_code, set_invite_code) = create_signal(String::new());
    let (wait, set_wait) = create_signal(false);

    let submit_action_resource = create_local_resource(
        move || (),
        move |_| async move {
            if wait.get_untracked()
                || email.get_untracked().is_empty()
                || password.get_untracked().is_empty()
                || invite_code.get_untracked().is_empty()
            {
                return;
            }
            set_wait.set(true);
            let credentials = RegisterForm {
                email: email.get_untracked(),
                password: password.get_untracked(),
                password_confirm: password.get_untracked(),
                invite_code: invite_code.get_untracked(),
            };
            let result = register(&state.blockmesh_url.get_untracked(), &credentials).await;
            match result {
                Ok(_) => {
                    state.api_token.update(|t| *t = uuid::Uuid::default());
                    send_message_channel(
                        MessageType::SET,
                        MessageKey::ApiToken,
                        Option::from(MessageValue::UUID(uuid::Uuid::default())),
                    )
                    .await;
                    state
                        .status
                        .update(|v| *v = AuthStatus::WaitingEmailVerification);
                    notifications.set_success("Please confirm email and login");
                }
                Err(err) => {
                    tracing::error!(
                        "Unable to register new account for {}: {err}",
                        credentials.email
                    );
                    notifications.set_error(err.to_string());
                }
            }
            set_wait.set(false);
        },
    );

    view! {
        <div class="auth-card">
            <img
                class="background-image"
                src="https://imagedelivery.net/3RKw_J_fJQ_4KpJP3_YgXA/0e42f33d-48e0-4f17-5ae0-9249a41bb200/public"
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
                    <div class="auth-card-input-container">
                        <input
                            required=""
                            type="text"
                            on:keyup=move |ev: ev::KeyboardEvent| {
                                let val = event_target_value(&ev);
                                set_invite_code.update(|v| *v = val);
                            }

                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                set_invite_code.update(|v| *v = val);
                            }
                        />

                        <label class="font-bebas-neue text-off-white">Refer Code</label>
                    </div>
                    <button
                        class="auth-card-button font-bebas-neue text-off-white"
                        on:click=move |_ev| {
                            submit_action_resource.refetch();
                        }
                    >

                        Register
                    </button>
                </form>
            </div>
            <div class="auth-card-bottom">
                <small class="font-open-sans text-orange">You already have an account?</small>
                <br/>
                <button on:click=move |_| { state.status.update(|v| *v = AuthStatus::LoggedOut) }>
                    <small class="text-magenta underline cursor-pointer">Login now</small>
                </button>
            </div>
        </div>
    }
}

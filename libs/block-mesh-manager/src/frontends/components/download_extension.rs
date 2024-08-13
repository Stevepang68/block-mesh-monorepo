use crate::frontends::components::icons::chrome_icon::ChromeIcon;
use crate::frontends::context::notification_context::NotificationContext;
use crate::frontends::context::webapp_context::WebAppContext;
use crate::frontends::frontend_webserver::components::error::ErrorComponent;
use crate::frontends::frontend_webserver::components::notification::NotificationComponent;
use block_mesh_common::constants::BLOCK_MESH_CHROME_EXTENSION_LINK;
use block_mesh_common::interfaces::server_api::CallToActionForm;
use block_mesh_common::routes_enum::RoutesEnum;
use leptos::Suspense;
use leptos::*;
use reqwest::Client;

#[component]
pub fn DownloadExtension(show: RwSignal<bool>) -> impl IntoView {
    let notifications = expect_context::<NotificationContext>();
    let async_data = WebAppContext::get_dashboard_data();
    let logged_in = WebAppContext::is_logged_in();
    let extension_installed = create_rw_signal(false);

    create_effect(move |_| {
        if let Some(Some(response)) = async_data.get() {
            if response
                .calls_to_action
                .iter()
                .any(|i| i.name == "install_extension")
            {
                extension_installed.set(true);
            }
        }
    });
    let submit = create_action(move |input: &String| {
        let input = input.clone();

        async move {
            let origin = window().origin();
            let client = Client::new();
            let response = client
                .post(format!(
                    "{}{}",
                    origin,
                    RoutesEnum::Static_Auth_Call_To_Action
                ))
                .form(&CallToActionForm {
                    name: "install_extension".to_string(),
                    status: true,
                })
                .send()
                .await;
            match response {
                Ok(_) => {
                    extension_installed.set(true);
                    notifications.set_success("Extension install status updated");
                    if input == "download" {
                        let _ = window()
                            .open_with_url_and_target(BLOCK_MESH_CHROME_EXTENSION_LINK, "_blank");
                    }
                }
                Err(_) => notifications.set_error("Failed to install status updated"),
            }
            show.set(false);
        }
    });

    view! {
        <Suspense fallback=move || {
            view! {
                <NotificationComponent
                    summary="Loading...".to_string()
                    detailed="Please wait while we load the dashboard".to_string()
                    go_to=RoutesEnum::Static_UnAuth_Login.to_string()
                />
            }
        }>
            <Show
                when=move || {
                    match logged_in.get() {
                        Some(Some(response)) => response.logged_in,
                        _ => false,
                    }
                }

                fallback=|| {
                    view! {
                        <ErrorComponent
                            code=401
                            summary="Not Logged In".to_string()
                            detailed="You must be logged in to view this page".to_string()
                            go_to=RoutesEnum::Static_UnAuth_Login.to_string()
                        />
                    }
                }
            >

                <div class="bg-dark-blue">
                    <form
                        action="/update_call_to_action"
                        method="post"
                        on:submit=|ev| ev.prevent_default()
                    >
                        <div class="m-2">
                            <div class="mb-4 rounded px-8 pb-8 pt-6 shadow-md bg-dark-blue flex justify-center">
                                <button
                                    class="inline-flex items-center justify-center text-cyan hover:text-magenta-2 py-2 px-4 border border-orange rounded font-bebas-neue focus:outline-none focus:shadow-outline"
                                    type="submit"
                                    on:click=move |_| {
                                        submit.dispatch("download".parse().unwrap())
                                    }
                                >

                                    <ChromeIcon/>
                                    Download Chrome extension and start earning
                                </button>
                            </div>
                            <div class="mb-4 rounded px-8 pb-8 pt-6 shadow-md bg-dark-blue flex justify-center">
                                <button
                                    class="inline-flex items-center justify-center text-off-white hover:text-magenta-2 py-2 px-4 border border-orange rounded font-bebas-neue focus:outline-none focus:shadow-outline"
                                    type="submit"
                                    on:click=move |_| {
                                        submit.dispatch("dismiss".parse().unwrap())
                                    }
                                >

                                    Already installed the extension
                                </button>
                            </div>
                        </div>
                    </form>
                </div>
            </Show>
        </Suspense>
    }
}

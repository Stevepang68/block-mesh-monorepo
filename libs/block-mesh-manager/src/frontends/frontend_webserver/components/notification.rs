use leptos::*;
use leptos_router::A;

#[component]
pub fn NotificationComponent(summary: String, detailed: String, go_to: String) -> impl IntoView {
    view! {
        <div class="bg-dark-blue flex justify-center items-center h-screen w-screen">
            <div class="bg-dark-blue border-off-white border-solid border-2 p-8 rounded-lg shadow-md m-2">
                <div class="text-center">
                    <h1 class="font-bebas-neue mt-4 text-3xl font-bold tracking-tight text-off-white sm:text-5xl">
                        {{ summary }}
                    </h1>
                    <p class="font-open-sans mt-6 text-base leading-7 text-off-white">
                        {{ detailed }}
                    </p>
                    <div class="mt-10 flex items-center justify-center gap-x-6">
                        <A
                            href=go_to
                            class="hover:text-orange text-off-white py-2 px-4 border border-orange rounded font-bebas-neue focus:outline-none focus:shadow-outline"
                        >

                            Go
                        </A>
                    </div>
                </div>
            </div>
        </div>
    }
}
